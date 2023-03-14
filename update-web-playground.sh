#!/usr/bin/env bash
nix build .#web-playground
ln -sf ../../result/post website/playground/wasm-app
