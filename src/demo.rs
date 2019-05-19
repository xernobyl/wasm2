// demo.rs

#![allow(dead_code)]

use wasm_bindgen::JsValue;
use crate::canvas3d::{ Canvas3D, Canvas3DCallbacks };
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::WebGl2RenderingContext;
use web_sys::console;
type GL = WebGl2RenderingContext;

use crate::framebuffer::Framebuffer;
use crate::shaders::ShaderManager;
use crate::static_geometry::StaticGeometry;
use crate::quad::Quad;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}


macro_rules! console_log {
	($($t:tt)*) => (console::log_1(&format_args!($($t)*).to_string()))
}


pub struct Demo {
	frame: u64,
	width: u32,
	height: u32,
	shader_manager: Option<ShaderManager>,
	static_geometry: Option<StaticGeometry>,
	framebuffer: Option<Framebuffer>,
	quad: Option<Quad>,
}


impl Demo {
	pub fn new() -> Self {
		Self {
			frame: 0,
			width: 0,
			height: 0,
			shader_manager: None,
			static_geometry: None,
			framebuffer: None,
			quad: None,
		}
	}

	pub fn run() -> Result<(), JsValue> {
		let demo = Demo::new();
		Canvas3D::run(Box::new(demo))
	}
}


impl Canvas3DCallbacks for Demo {
	fn init(&mut self, gl: &GL) -> Result<(), String> {
		log("Demo::init");
		let mut static_geometry = StaticGeometry::new(&gl, 16 * 1024 * 1024, 2 * 1024 * 1024)?;

		self.quad = Some(Quad::new(&gl, &mut static_geometry));
		self.static_geometry = Some(static_geometry);
		self.framebuffer = Some(Framebuffer::new(&gl, self.width, self.height)?);

		Ok(())
	}


	fn frame(&mut self, gl: &GL, timestamp: f64) {
		gl.clear_color(
			(f64::sin(0.0028564 * timestamp + 0.956564) * 0.5 + 0.5) as f32,
			(f64::sin(0.0034542 * timestamp + 1.4566564) * 0.5 + 0.5) as f32,
			(f64::sin(0.001436376 * timestamp + 0.1283746876) * 0.5 + 0.5) as f32,
			1.0);
		gl.clear(GL::COLOR_BUFFER_BIT);
	}

	fn resize(&mut self, width: u32, height: u32) {
		log(&format!("Demo::resize {} {}", width, height));
		self.width = width;
		self.height = height;
	}
}
