use crate::file_definition::FileID;
use crate::ir::variable_meta::VariableMeta;
use crate::ssa::dominator_tree::DominatorTree;
use crate::ssa::errors::SSAResult;
use crate::ssa::traits::{DirectedGraphNode, Version};
use crate::ssa::{insert_phi_statements, insert_ssa_variables};

use super::basic_block::BasicBlock;
use super::param_data::ParameterData;
use super::ssa_impl::VersionEnvironment;

/// Basic block index type.
type Index = usize;

pub struct CFG {
    name: String,
    param_data: ParameterData,
    basic_blocks: Vec<BasicBlock>,
    dominator_tree: DominatorTree<BasicBlock>,
}

impl CFG {
    pub(crate) fn new(
        name: String,
        param_data: ParameterData,
        basic_blocks: Vec<BasicBlock>,
        dominator_tree: DominatorTree<BasicBlock>,
    ) -> CFG {
        CFG {
            name,
            param_data,
            basic_blocks,
            dominator_tree,
        }
    }
    /// Returns the entry (first) block of the CFG.
    pub fn get_entry_block(&self) -> &BasicBlock {
        &self.basic_blocks[Index::default()]
    }
    /// Returns the number of basic blocks in the CFG.
    pub fn nof_basic_blocks(&self) -> usize {
        self.basic_blocks.len()
    }
    /// Convert the CFG into SSA form.
    pub fn into_ssa(&mut self) -> SSAResult<()> {
        for basic_block in self.iter_mut() {
            basic_block.cache_variable_use();
        }
        let mut env: VersionEnvironment = self.get_parameters().into();
        insert_phi_statements(&mut self.basic_blocks, &self.dominator_tree);
        insert_ssa_variables(&mut self.basic_blocks, &self.dominator_tree, &mut env)?;
        for name in self.param_data.iter_mut() {
            *name = name.with_version(Version::default());
        }
        for basic_block in self.iter_mut() {
            basic_block.cache_variable_use();
        }
        Ok(())
    }
    /// Get the name of the corresponding function or template.
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Get the file ID for the corresponding function or template.
    pub fn get_file_id(&self) -> FileID {
        self.param_data.get_file_id()
    }
    /// Returns the parameter data for the corresponding function or template.
    pub fn get_parameters(&self) -> &ParameterData {
        &self.param_data
    }
    /// Returns an iterator over the basic blocks in the CFG.
    pub fn iter(&self) -> impl Iterator<Item = &BasicBlock> {
        self.basic_blocks.iter()
    }
    /// Returns a mutable iterator over the basic blocks in the CFG.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BasicBlock> {
        self.basic_blocks.iter_mut()
    }
    /// Returns the dominators of the given basic block
    pub fn get_dominators(&self, basic_block: &BasicBlock) -> Vec<&BasicBlock> {
        self.dominator_tree
            .get_dominators(basic_block.get_index())
            .iter()
            .map(|&i| &self.basic_blocks[i])
            .collect()
    }
    /// Returns the immediate dominator of the basic block (that is, the
    /// predecessor of the node in the CFG dominator tree), if it exists.
    pub fn get_immediate_dominator(&self, basic_block: &BasicBlock) -> Option<&BasicBlock> {
        self.dominator_tree
            .get_immediate_dominator(basic_block.get_index())
            .map(|i| &self.basic_blocks[i])
    }
    /// Get immediate successors of the basic block in the CFG dominator tree.
    /// (For a definition of the dominator relation, see `CFG::get_dominators`.)
    pub fn get_dominator_successors(&self, basic_block: &BasicBlock) -> Vec<&BasicBlock> {
        self.dominator_tree
            .get_dominator_successors(basic_block.get_index())
            .iter()
            .map(|&i| &self.basic_blocks[i])
            .collect()
    }
    /// Returns the dominance frontier of the basic block. The _dominance
    /// frontier_ of `i` is defined as all basic blocks `j` such that `i`
    /// dominates an immediate predecessor of `j`, but i does not strictly
    /// dominate `j`. (`j` is where `i`s dominance ends.)
    pub fn get_dominance_frontier(&self, basic_block: &BasicBlock) -> Vec<&BasicBlock> {
        self.dominator_tree
            .get_dominance_frontier(basic_block.get_index())
            .iter()
            .map(|&i| &self.basic_blocks[i])
            .collect()
    }
}