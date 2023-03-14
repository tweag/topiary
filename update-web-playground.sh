#!/usr/bin/env bash
cargo build --lib --target wasm32-unknown-unknown --manifest-path=web-playground/Cargo.toml
wasm-bindgen --target web --out-dir website/playground/wasm-app web-playground/target/wasm32-unknown-unknown/debug/topiary_playground.wasm
