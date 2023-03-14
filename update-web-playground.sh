#!/usr/bin/env bash
cargo build --manifest-path=web-playground/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir website/playground/wasm-app web-playground/target/wasm32-unknown-unknown/debug/topiary_playground.wasm
