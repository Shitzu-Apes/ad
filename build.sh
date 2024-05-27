#!/bin/bash
set -e
cd "`dirname $0`"

cargo build --release -p ad-token --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ./res/

wasm-opt -O4 res/ad_token.wasm -o res/ad_token.wasm --strip-debug --vacuum
