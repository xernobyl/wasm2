// canvas3d.rs

//use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

//use web_sys::WebGl2RenderingContext;
use web_sys::*;

use std::cell::RefCell;
use std::rc::Rc;


type GL = WebGl2RenderingContext;


#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}


pub struct Canvas3D {
	gl: Rc<GL>,
	canvas: Rc<HtmlCanvasElement>,
	callbacks: Canvas3DCallbacks,
}


impl Canvas3D {
	pub fn run(callbacks: &'static mut Canvas3DCallbacks) -> Result<(), JsValue> {
		let (canvas, gl) = Self::create_canvas_element().map_err(Self::js_to_str)?;
		let canvas = Rc::new(canvas);

		/*{
			let canvas = canvas.clone();
			let closure = Closure::wrap(Box::new(move || {
				let width = canvas.client_width() as u32;
				let height = canvas.client_height() as u32;

				if width != 0 && height != 0 {
					log(format!("Resizing: {} * {}", width, height).as_ref());
					canvas.set_width(width);
					canvas.set_height(height);

					callbacks.resize(width, height);
				}
			}) as Box<dyn FnMut()>);

			window().unwrap().set_onresize(Option::Some(closure.as_ref().unchecked_ref()));
			closure.forget();
    }*/

		log(format!("Starting loop...").as_ref());

		unsafe {
			window()
			.unwrap()
			.request_animation_frame(std::mem::transmute(request_animation_frame_callback));
		}

		Ok(())
	}


	fn js_to_str(value: JsValue) -> String {
		value.as_string().unwrap_or_else(|| "???".to_string())
	}


	fn create_canvas_element() -> Result<(HtmlCanvasElement, GL), JsValue> {
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

			let gl = canvas.get_context("webgl2")?.unwrap().dyn_into::<GL>()?;

			Result::Ok((canvas, gl))
		}
}


pub trait Canvas3DCallbacks {
	fn frame(&mut self, gl: &GL, timestamp: f64);
	fn resize(&mut self, width: u32, height: u32);
}


static mut current_canvas3d: *mut Canvas3D = std::mem::transmute(0);


// https://developer.mozilla.org/en-US/docs/Games/Anatomy
unsafe fn request_animation_frame_callback(timestamp: f64) {
	unsafe {
		window()
		.unwrap()
		.request_animation_frame(std::mem::transmute(request_animation_frame_callback));
	}
	(*current_canvas3d).callbacks.frame((*current_canvas3d).gl, timestamp);
}


fn request_animation_frame(f: &Closure<FnMut(f64)>) {
	window().unwrap()
		.request_animation_frame(f.as_ref().unchecked_ref())
		.expect("no requestAnimationFrame");
}
