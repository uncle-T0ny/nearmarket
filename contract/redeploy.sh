#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release  \
        && NEAR_ENV=mainnet near deploy --accountId archimarket.near --wasmFile target/wasm32-unknown-unknown/release/archimarket.wasm \