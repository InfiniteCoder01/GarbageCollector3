#!/usr/bin/env bash
source assets/world-codegen.sh
source assets/player-skin.sh
cargo build --target=wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/*.wasm --out-dir html --target web
python -m http.server -d html
