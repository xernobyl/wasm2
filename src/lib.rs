extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;

#[macro_use]
mod utils;

mod app;
mod half_cube;
mod fullscreen_buffers;
mod fast_rand;
mod line_2d_strip;
pub mod scene;


#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
  app::App::init();
  Ok(())
}
