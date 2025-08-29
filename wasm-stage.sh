#!/usr/bin/env bash

# safer script (exit on error, unset vars, and pipe errors).
set -euo pipefail

RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
    wasm-pack build --target web --no-default-features

rm -rf ./docs

mkdir -p ./docs

# Copy everything from pkg to docs, excluding .gitignore
shopt -s extglob

cp -r ./pkg/!(.gitignore) ./docs/

shopt -u extglob
