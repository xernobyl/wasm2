#![feature(maybe_uninit)]
#![feature(const_transmute)]

use wasm_bindgen::prelude::*;

//pub mod gl_web;
//pub mod gl_sys;
//pub mod shaders;
//pub mod static_geometry;
//pub mod cube;
//pub mod framebuffer;
pub mod canvas3d;
pub mod demo;

//use self::gl_web::GLWeb;
//use self::gl_sys::GLSys;


#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn web_main() -> Result<(), JsValue> {
	/*match GLWeb::new() {
		Err(msg) => {
			log(&msg);
			Err(JsValue::from(msg))
		},
		Ok(gl) => {
			gl.start_loop();
			Ok(())
		}
	}*/

	demo::Demo::run()
}
