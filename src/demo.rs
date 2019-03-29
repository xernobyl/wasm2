// demo.rs

use wasm_bindgen::JsValue;
use crate::canvas3d::{ Canvas3D, Canvas3DCallbacks };
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::WebGl2RenderingContext;
type GL = WebGl2RenderingContext;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}


pub struct Demo {
}


impl Demo {
	pub fn run() -> Result<(), JsValue> {
		static mut DEMO: Demo = Demo {
		};

		Canvas3D::run(unsafe { &mut DEMO });

		Ok(())
	}
}


impl Canvas3DCallbacks for Demo {
	fn frame(&mut self, gl: &GL, _timestamp: f64) {
		log("Demo::frame");
	}

	fn resize(&mut self, width: u32, height: u32) {
		log(&format!("Demo::resize {} {}", width, height));
	}
}
