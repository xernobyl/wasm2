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
	//gl: GL,
	//canvas: HtmlCanvasElement,
}


fn request_animation_frame(f: &Closure<FnMut(f64)>) {
	window().unwrap().request_animation_frame(f.as_ref().unchecked_ref());
}


impl Canvas3D {
	pub fn run(callbacks: Box<Canvas3DCallbacks>) -> Result<(), JsValue> {
		let (canvas, gl) = Self::create_canvas_element().map_err(Self::js_to_str)?;

		{
			let closure = Closure::wrap(Box::new(move || {
				let width = canvas.client_width() as u32;
				let height = canvas.client_height() as u32;

				if width != 0 && height != 0 {
					log(format!("Resizing: {} * {}", width, height).as_ref());
					canvas.set_width(width);
					canvas.set_height(height);

					//callbacks.resize(width, height);
				}
			}) as Box<dyn FnMut()>);

			window().unwrap().set_onresize(Option::Some(closure.as_ref().unchecked_ref()));
			closure.forget();
    }

		let callbacks = Box::leak(callbacks);
		
		{
			// https://developer.mozilla.org/en-US/docs/Games/Anatomy
			let f: Rc<_> = Rc::new(RefCell::new(None));
			let g = f.clone();

			let closure = Some(Closure::wrap(Box::new(move |timestamp| {
				request_animation_frame(f.borrow().as_ref().unwrap());

				callbacks.frame(&gl, timestamp);
			}) as Box<dyn FnMut(_)>));

			*g.borrow_mut() = closure;
			request_animation_frame(g.borrow().as_ref().unwrap());
		}

		log(format!("Starting loop...").as_ref());

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
