use wasm_bindgen::prelude::*;

#[macro_use]
mod utils;

mod app;
mod camera;
mod chunk;
mod demo;
mod gpu;
mod projection;
mod ecs;
mod fast_rand;
mod half_cube;
mod line_2d_strip;
mod particles;
mod scene;
mod scene1;
mod stereo_camera;
mod view;
#[cfg(target_arch = "wasm32")]
mod xr;

use crate::app::App;
use crate::demo::Demo;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let demo = Demo::new();
    App::init(Box::new(demo));
    Ok(())
}
