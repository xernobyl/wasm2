use wasm_bindgen::prelude::*;

pub mod shaders;
pub mod static_geometry;
pub mod quad;
pub mod cube;
pub mod framebuffer;
pub mod canvas3d;
pub mod demo;


#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn web_main() -> Result<(), JsValue> {
	demo::Demo::run()
}
