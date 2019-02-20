use wasm_bindgen::prelude::*;

pub mod gl_web;
pub mod gl_sys;
pub mod shaders;
pub mod static_geometry;

use self::gl_web::GLWeb;
use self::gl_sys::GLSys;


#[wasm_bindgen(start)]
pub fn web_main() -> Result<(), JsValue> {
	let gl = GLWeb::new()?;
	gl.start_loop();
	Ok(())
}
