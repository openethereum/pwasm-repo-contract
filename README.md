[![Build Status](https://travis-ci.org/paritytech/pwasm-token-example.svg?branch=master)](https://travis-ci.org/paritytech/pwasm-repo-contract)
## Description
A simple contract allows to lend some amount of particular ERC20 token (impl by https://github.com/paritytech/pwasm-token-example) for some interest. It demonstrates how WASM contracts can depend on each other and communicate through ABI. It shows also how to mock callee contract in the test environment.
## Build prerequisites
- rust with `wasm32-unknown-emscripten` target - instruction to setup can be found [here](https://hackernoon.com/compiling-rust-to-webassembly-guide-411066a69fde)
- make sure `emcc` tool is in the `PATH` since build script uses it internally
- wasm build util, run `cargo install --git https://github.com/paritytech/wasm-utils wasm-build` to install
- bash to run `./build.sh`
## Build
`./build.sh`
## Testing
`cargo test --manifest-path="contract/Cargo.toml" --features std`
