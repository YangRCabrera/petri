#!/usr/bin/env bash
# Vercel's build image doesn't ship Rust or wasm-pack, so install them
# here before handing off to the normal `npm run build` pipeline
# (build:wasm, then web's tsc + vite build).
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
  source "$HOME/.cargo/env"
fi

rustup target add wasm32-unknown-unknown

if ! command -v wasm-pack >/dev/null 2>&1; then
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

npm run build
