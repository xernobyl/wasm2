pub mod gl_web;
pub mod gl_sys;

use wasm_bindgen::prelude::*;

use self::gl_web::GLWeb;
use self::gl_sys::GLSys;


#[wasm_bindgen(start)]
pub fn web_main() -> Result<(), JsValue> {
	let gl = GLWeb::new()?;
	gl.start_loop();
	Ok(())
}
