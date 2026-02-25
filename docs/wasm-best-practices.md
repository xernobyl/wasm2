# Rust WebAssembly best practices (this project)

## What you're doing right

- **JS entry**: `(async () => await init())();` in HTML is correct — `wasm-bindgen`'s `init()` returns a Promise; awaiting it from JS is the recommended way to load WASM.
- **Panic hook**: `console_error_panic_hook::set_once()` in `main()` so panics show up in the console.
- **`main()` signature**: `Result<(), JsValue>` and `Ok(())` so JS can see errors.
- **rAF loop**: The `Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>` pattern is the standard, idiomatic way to have a closure schedule itself with `request_animation_frame` — the closure must own (or hold a ref to) the next callback so it isn’t dropped before the next frame.
- **Long‑lived listeners**: Calling `closure.forget()` after `add_event_listener_with_callback` / `set_onresize` is correct so the closure isn’t dropped and the listener stays valid.
- **Sync init**: Doing WebGL setup and starting the rAF loop synchronously in `App::init` is fine; you don’t need async Rust for the main loop.

## Promises / futures

- **When to use**: Use `wasm_bindgen_futures` when you have one-off async work from Rust (e.g. `fetch`, `navigator.xr.is_session_supported()`). You already have the dependency; the commented-out `JsFuture::from(vr_supported)` is the right pattern for WebXR.
- **rAF**: You don’t need to turn the render loop into async/await. The closure-based rAF loop is normal and avoids the complexity of a recursive `async fn` + `spawn_local`/`then`.
- **Init**: Keeping Rust `main()` synchronous and doing “async load” only on the JS side (`await init()`) is a common and good approach.

## Event handling and “Rustfulness”

- **Typed events**: Right now keydown/keyup use `FnMut()` so you don’t get the event. For real input (key code, `prevent_default`, etc.) use the typed API and pass the event into Rust:
  - Callback: `Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| { ... }) as Box<dyn FnMut(_)>).as_ref().unchecked_ref()`.
  - Enable the `KeyboardEvent` (and optionally `MouseEvent`, `WheelEvent`) features in `web-sys` in `Cargo.toml` if you use them.
- **Errors**: Prefer `.expect("message")` or `?` instead of `#[allow(unused_must_use)]` where you can, so listener and rAF failures are visible.
- **Clean shutdown**: To stop the rAF loop (e.g. for VR or unloading), do `f.borrow_mut().take()` so the stored `Closure` is dropped and no more frames are scheduled.

## Optional improvements

- **Resize**: Debouncing resize (e.g. only applying `new_width`/`new_height` inside the rAF callback) avoids multiple resize updates per frame; you can keep a “pending resize” flag and apply it once per frame (you already do something similar).
- **VR**: When you re-enable WebXR, use `JsFuture::from(navigator.xr().is_session_supported(...))` and `wasm_bindgen_futures::spawn_local(async move { ... })` (or a small async block) instead of `js_sys::eval`, so you stay in typed, promise-based Rust.

Summary: Your use of promises (on the JS side), the rAF closure pattern, and `closure.forget()` for listeners is correct and idiomatic. The main upgrades are typing keyboard (and other) events and using futures for one-off async APIs like WebXR when you need them.
