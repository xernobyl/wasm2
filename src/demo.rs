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
	val: u32,
}


impl Demo {
	pub fn new() -> Self {
		Self {
			val: 0,
		}
	}

	pub fn run() -> Result<(), JsValue> {
		let demo = Demo::new();
		Canvas3D::run(Box::new(demo))
	}
}


impl Canvas3DCallbacks for Demo {
	fn frame(&mut self, gl: &GL, timestamp: f64) {
		gl.clear_color(
			(f64::sin(0.0028564 * timestamp + 0.956564) * 0.5 + 0.5) as f32,
			(f64::sin(0.0034542 * timestamp + 1.4566564) * 0.5 + 0.5) as f32,
			(f64::sin(0.001436376 * timestamp + 0.1283746876) * 0.5 + 0.5) as f32,
			1.0);
		gl.clear(GL::COLOR_BUFFER_BIT);
		self.val += 1;
	}

	fn resize(&mut self, width: u32, height: u32) {
		log(&format!("Demo::resize {} {}", width, height));
	}
}
  