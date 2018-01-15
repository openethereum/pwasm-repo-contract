#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-build ./target pwasm_repo_bin --target=wasm32-unknown-unknown --final=repo --save-raw=./target/repo-deployed.wasm

cp ./target/*.wasm ./compiled
cp ./target/json/* ./compiled
