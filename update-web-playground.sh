#!/usr/bin/env bash
nix build .#web-playground
rm -rf website/playground/wasm-app
ln -s ../../result/post website/playground/wasm-app
