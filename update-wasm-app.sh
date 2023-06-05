#!/usr/bin/env bash
nix build .#topiary-playground
rm -rf web-playground/src/wasm-app
mkdir web-playground/src/wasm-app
cp -r result/* web-playground/src/wasm-app/
rm -rf web-playground/src/samples
mkdir web-playground/src/samples
mv web-playground/src/wasm-app/languages_export.ts web-playground/src/samples/
