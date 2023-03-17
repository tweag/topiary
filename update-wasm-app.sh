#!/usr/bin/env bash
nix build .#wasm-app
rm -rf web-playground/react-app/src/wasm-app
cp -r result web-playground/react-app/src/wasm-app
chmod -R +w web-playground/react-app/src/wasm-app
