pub mod gl_web;
pub mod gl_sys;

use wasm_bindgen::prelude::*;

use self::gl_web::GLWeb;


#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}


#[wasm_bindgen(start)]
pub fn web_main() -> Result<(), JsValue> {
	let gl = GLWeb::new()?;
	gl.start_loop();
	Ok(())
}
