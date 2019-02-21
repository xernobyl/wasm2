/*
Useful example for callbacks and stuff here:
https://rustwasm.github.io/wasm-bindgen/examples/paint.html?highlight=create_element#srclibrs
https://rustwasm.github.io/wasm-bindgen/api/js_sys/
https://rustwasm.github.io/wasm-bindgen/api/web_sys/
*/

#![allow(dead_code)]

use crate::shaders::ShaderManager;
use crate::gl_sys::GLSys;
use crate::static_geometry::StaticGeometry;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

use std::cell::RefCell;
use std::rc::Rc;

use web_sys::*;


const FRAGMENT_SHADER_0: &str = r#"
void main() {
	gl_FragColor = vec4(1.0, 0.5, 0.25, 1.0);
}
"#;


const VERTEX_SHADER_0: &str = r#"
attribute vec4 position;
void main() {
	gl_Position = position;
}
"#;


#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}


pub struct GLWeb {
	canvas: Rc<HtmlCanvasElement>,
	gl: Rc<WebGl2RenderingContext>,
	frame: u64,
	shader_manager: ShaderManager,
	static_geometry: StaticGeometry,
}


impl GLWeb {
	fn frame_callback(&mut self, frame_time: f64) {
		const GOLDEN_RATIO: f64 = 1.618_033_988_749_895;
		self.gl.clear_color(
			0.0 + (frame_time * GOLDEN_RATIO).fract() as f32,
			0.25 + (frame_time * GOLDEN_RATIO).fract() as f32,
			0.5 + (frame_time * GOLDEN_RATIO).fract() as f32,
			0.75 + (frame_time * GOLDEN_RATIO).fract() as f32);

		self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

		self.shader_manager.bind_shader(&self.gl, 0);

		self.gl.draw_arrays(
			WebGl2RenderingContext::TRIANGLES,
			0,
			(9 / 3) as i32,
		);

		self.frame += 1;
	}
}


impl GLSys for GLWeb {
	fn new() -> Result<Self, String> where Self: Sized {
		fn create_canvas_element() -> Result<(HtmlCanvasElement, WebGl2RenderingContext), JsValue> {
			let document = window().ok_or("Can't get window")?
				.document().ok_or("Can't get document")?;

			let canvas = document
				.create_element("canvas")?
				.dyn_into::<HtmlCanvasElement>()?;

			canvas.style().set_property("position", "fixed")?;
			canvas.style().set_property("left", "0")?;
			canvas.style().set_property("top", "0")?;
			canvas.style().set_property("width", "100%")?;
			canvas.style().set_property("height", "100%")?;

			document.body().unwrap().append_child(&canvas)?;

			let width = canvas.client_width() as u32;
			let height = canvas.client_height() as u32;

			if width != 0 && height != 0 {
				canvas.set_width(width);
				canvas.set_height(height);
			}

			let gl = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

			Result::Ok((canvas, gl))
		}

		fn js_to_str(value: JsValue) -> String {
			value.as_string().unwrap_or_else(|| "???".to_string())
		}

		let (canvas, gl) = create_canvas_element().map_err(js_to_str)?;
		let canvas = Rc::new(canvas);
		let gl = Rc::new(gl);

		{
			let canvas = canvas.clone();
			let closure = Closure::wrap(Box::new(move || {
				let width = canvas.client_width() as u32;
				let height = canvas.client_height() as u32;

				if width != 0 && height != 0 {
					log(format!("Resizing: {} * {}", width, height).as_ref());

					canvas.set_width(width);
					canvas.set_height(height);
				}
			}) as Box<dyn FnMut()>);

			window().unwrap().set_onresize(Option::Some(closure.as_ref().unchecked_ref()));
			closure.forget();
    }

		let mut shader_manager = ShaderManager::new(/* &gl */);
		shader_manager.create_shader(&gl, VERTEX_SHADER_0, FRAGMENT_SHADER_0)?;

		let static_geometry = StaticGeometry::new(&gl, 16 * 1024 * 1024)?;

		///////////////////////////////////

		let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
		let memory_buffer = wasm_bindgen::memory()
		    .dyn_into::<WebAssembly::Memory>().map_err(js_to_str)?
		    .buffer();
		let vertices_location = vertices.as_ptr() as u32 / 4;
		let vert_array = js_sys::Float32Array::new(&memory_buffer)
		    .subarray(vertices_location, vertices_location + vertices.len() as u32);

		let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
		gl.buffer_data_with_array_buffer_view(
			WebGl2RenderingContext::ARRAY_BUFFER,
			&vert_array,
			WebGl2RenderingContext::STATIC_DRAW,
		);
		gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
		gl.enable_vertex_attrib_array(0);

		///////////////////////////////////

		Result::Ok(GLWeb {
			canvas,
			gl,
			frame: 0,
			shader_manager,
			static_geometry,
		})
	}

	fn start_loop(self) {
		fn request_animation_frame(f: &Closure<FnMut(f64)>) {
			window().unwrap()
				.request_animation_frame(f.as_ref().unchecked_ref())
				.expect("no requestAnimationFrame");
		}

		log(format!("Starting loop...").as_ref());

		let mut rc = Rc::new(self);
		let f = Rc::new(RefCell::new(None));
		let g = f.clone();

		let closure = Some(Closure::wrap(Box::new(move |timestamp| {
			if let Some(the_self) = Rc::get_mut(&mut rc) {
				the_self.frame_callback(timestamp);
			};
			request_animation_frame(f.borrow().as_ref().unwrap());
		}) as Box<dyn FnMut(_)>));

		*g.borrow_mut() = closure;
		request_animation_frame(g.borrow().as_ref().unwrap());
		// closure.forget();
	}
}
