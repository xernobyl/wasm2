extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;

#[macro_use]
mod utils;

mod app;
mod demo;
mod fast_rand;
mod fullscreen_buffers;
mod half_cube;
mod line_2d_strip;
mod scene;
mod scene1;
mod shaders;
mod particles;

use crate::app::App;
use crate::demo::Demo;


#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let demo = Demo::new();
    let app = App::init(Box::new(demo));
    Ok(())
}
