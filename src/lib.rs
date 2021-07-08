extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;

#[macro_use]
mod utils;

mod app;
mod fast_rand;
mod fullscreen_buffers;
mod half_cube;
mod line_2d_strip;
mod scene;
mod scene1;
mod shaders;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    app::App::init();
    Ok(())
}
