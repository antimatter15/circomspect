FROM rust:latest as build-stage
RUN curl https://raw.githubusercontent.com/second-state/rustwasmc/master/installer/init.sh -sSf | sh
RUN wget https://github.com/WebAssembly/binaryen/releases/download/version_111/binaryen-version_111-x86_64-linux.tar.gz && tar -xvf binaryen-version_111-x86_64-linux.tar.gz -C / --strip-components=1
RUN rustup update nightly && rustup default nightly
COPY . /src
RUN rustwasmc build --enable-aot /src/cli
FROM scratch
COPY --from=build-stage /src/cli/pkg/circomspect.wasm /

