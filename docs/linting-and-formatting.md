# Linting and static checking

## Toolchain and dependencies (2026)

- **Rust**: `edition = "2024"`, `rust-version = "1.85"` (required for edition 2024).
- **Dependencies**: Pinned to current patch versions (e.g. wasm-bindgen 0.2.110, js-sys / web-sys 0.3.87, wasm-bindgen-futures 0.4.60, serde 1.0.x, serde-wasm-bindgen 0.6.5, console_error_panic_hook 0.1.7). Run `cargo update` to refresh within semver.

## What’s configured

### Safety and correctness

- **rustc** (`[lints.rust]`): `unsafe_op_in_unsafe_fn`, `unused_imports`, `unused_crate_dependencies` → warn.
- **Clippy** (`[lints.clippy]`): `all`, `pedantic`, `nursery` at **warn**, plus explicit **warn** for:
  - **Safety**: `undocumented_unsafe_blocks`
  - **Performance**: `redundant_clone`, `unnecessary_lazy_evaluations`, `needless_borrow`, `single_char_add_str`
  - **Correctness**: `needless_question_mark`, `expect_fun_call`, `option_if_let_else`  
  A few lints are `allow` where they’re noisy or not applicable (casts, docs, `result_unit_err`, etc.).

### Performance (release builds)

- **`[profile.release]`** is set for smaller, faster WASM:
  - `opt-level = 3`, `lto = true`, `codegen-units = 1` for maximum optimization.
  - `panic = "abort"` for smaller binary (no unwind tables); don’t use `catch_unwind` with this.
  - `strip = true` to remove symbols from the built artifact.

### Formatting

- **rustfmt**: `rustfmt.toml` at repo root (max line width 100, 4 spaces, Unix newlines).

## Commands

Install the wasm target if needed:

```bash
rustup target add wasm32-unknown-unknown
```

Then:

| Task | Command |
|------|--------|
| Lint (WASM target) | `cargo lint` (alias) or `cargo clippy --target wasm32-unknown-unknown` |
| Format code | `cargo fmt` |
| Check format only | `cargo fmt-check` (alias) or `cargo fmt -- --check` |
| Check (no build artifacts) | `cargo check --target wasm32-unknown-unknown` |

Use `cargo lint` and `cargo fmt-check` in CI to enforce static checks and formatting.

## Relaxing or tightening lints

- To **treat Clippy warnings as errors** (e.g. in CI):  
  `RUSTFLAGS="-D warnings" cargo clippy --target wasm32-unknown-unknown`
- To **tone down**: change `pedantic` / `nursery` (or specific lints) from `"warn"` to `"allow"` in `Cargo.toml` under `[lints.clippy]`.
- To **enable more**: add or flip lints in `[lints.clippy]`; see [Clippy lints](https://doc.rust-lang.org/clippy/lints/index.html).
