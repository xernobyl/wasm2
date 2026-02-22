#!/usr/bin/env bash
set -e
# Prevent rustup "unknown proxy name" when run from Cursor/IDE: zsh and some
# environments set ARGV0 to the app name, which rustup treats as the proxy name.
unset ARGV0
PATH="${HOME}/.cargo/bin:${PATH}"
if ! command -v wasm-pack &>/dev/null; then
	echo "wasm-pack not found. Install it with:"
	echo "  cargo install wasm-pack"
	echo "Or: https://rustwasm.github.io/wasm-pack/installer/"
	exit 1
fi
wasm-pack build --target web --dev --no-typescript
