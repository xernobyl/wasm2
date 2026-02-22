#!/usr/bin/env bash
set -e
PATH="${HOME}/.cargo/bin:${PATH}"
if ! command -v wasm-pack &>/dev/null; then
	echo "wasm-pack not found. Install it with:"
	echo "  cargo install wasm-pack"
	echo "Or: https://rustwasm.github.io/wasm-pack/installer/"
	exit 1
fi
wasm-pack build --target web --release --no-typescript
