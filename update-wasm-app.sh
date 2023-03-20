#!/usr/bin/env bash
nix build .#topiary-playground
rm -rf web-playground/src/wasm-app
cp -r result web-playground/src/wasm-app
chmod -R +w web-playground/src/wasm-app
