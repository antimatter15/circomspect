use crate::errors::FileOsError;

use super::errors::IncludeError;
use program_structure::ast::Include;
use program_structure::report::{Report, ReportCollection};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::path::Component;



// Replacement for std::fs::canonicalize that doesn't verify the path exists
// Plucked from https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
// Advice from https://www.reddit.com/r/rust/comments/hkkquy/comment/fwtw53s/?utm_source=share&utm_medium=web2x&context=3
fn normalize_path(path: &PathBuf) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}


pub struct FileStack {
    current_location: Option<PathBuf>,
    black_paths: HashSet<PathBuf>,
    user_inputs: HashSet<PathBuf>,
    stack: Vec<PathBuf>,
}

impl FileStack {
    pub fn new(paths: &[PathBuf], reports: &mut ReportCollection) -> FileStack {
        let mut result = FileStack {
            current_location: None,
            black_paths: HashSet::new(),
            user_inputs: HashSet::new(),
            stack: Vec::new(),
        };
        result.add_files(paths, reports);
        result.user_inputs = result.stack.iter().cloned().collect::<HashSet<_>>();

        result
    }

    fn add_files(&mut self, paths: &[PathBuf], reports: &mut ReportCollection) {
        for path in paths {
            if path.is_dir() {
                // Handle directories on a best effort basis only.
                let mut paths = Vec::new();
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        paths.push(entry.path())
                    }
                }
                self.add_files(&paths, reports);
            } else if let Some(extension) = path.extension() {
                // Add Circom files to file stack.
                if extension == "circom" {
                    let path = normalize_path(&path);
                    if path.is_file() {
                        self.stack.push(path);
                    } else {
                        reports.push(
                            FileOsError { path: path.display().to_string() }.into_report(),
                        );
                    }
                }
            }
        }
    }

    pub fn add_include(&mut self, include: &Include) -> Result<(), Box<Report>> {
        let mut location = self.current_location.clone().expect("parsing file");
        location.push(include.path.clone());
        let path = normalize_path(&location);
        if path.is_file() {
            if !self.black_paths.contains(&path) {
                self.stack.push(path);
            }
            Ok(())
        } else {
            let error = IncludeError {
                path: include.path.clone(),
                file_id: include.meta.file_id,
                file_location: include.meta.file_location(),
            };
            Err(Box::new(error.into_report()))
        }
    }

    pub fn take_next(&mut self) -> Option<PathBuf> {
        loop {
            match self.stack.pop() {
                None => {
                    break None;
                }
                Some(file_path) if !self.black_paths.contains(&file_path) => {
                    let mut location = file_path.clone();
                    location.pop();
                    self.current_location = Some(location);
                    self.black_paths.insert(file_path.clone());
                    break Some(file_path);
                }
                _ => {}
            }
        }
    }

    pub fn is_user_input(&self, path: &PathBuf) -> bool {
        self.user_inputs.contains(path)
    }
}
