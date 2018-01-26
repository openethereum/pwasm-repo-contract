[![Build Status](https://travis-ci.org/paritytech/pwasm-token-example.svg?branch=master)](https://travis-ci.org/paritytech/pwasm-repo-contract)
## Description
A simple contract allows to lend some amount of particular ERC20 token (impl by https://github.com/paritytech/pwasm-token-example) for some interest. It demonstrates how WASM contracts can depend on each other and communicate through ABI. It shows also how to mock callee contract in the test environment.
## Build prerequisites
Install rust with `wasm32-unknown-unknown` target:
```
rustup target add wasm32-unknown-unknown --toolchain nightly
```
Install Wasm build util:
```
cargo install --git https://github.com/paritytech/wasm-utils wasm-build
```
## Build
Run:
```
./build.sh
```
## Testing
```
cargo test --manifest-path="contract/Cargo.toml" --features std
```
