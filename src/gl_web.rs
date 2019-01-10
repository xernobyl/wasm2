use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader, HtmlCanvasElement};
use std::cell::RefCell;
use std::rc::Rc;
use rand::Rng;

use gl_sys;


const FRAGMENT_SHADER_0: &str = r#"
void main() {
	gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
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


fn request_animation_frame(f: &Closure<FnMut()>) {
	web_sys::window().unwrap()
		.request_animation_frame(f.as_ref().unchecked_ref())
		.expect("should register `requestAnimationFrame` OK");
}


fn draw_frame(gl: &WebGl2RenderingContext, frame: &usize) {
	const GOLDEN_RATIO: f64 = 1.6180339887498948420;
	// let mut rng = rand::thread_rng();
	// gl.clear_color(rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0));
	gl.clear_color(
		(0.0 + f64::from(*frame as u32) * GOLDEN_RATIO).fract() as f32,
		(0.25 + f64::from(*frame as u32) * GOLDEN_RATIO).fract() as f32,
		(0.5 + f64::from(*frame as u32) * GOLDEN_RATIO).fract() as f32,
		(0.75 + f64::from(*frame as u32) * GOLDEN_RATIO).fract() as f32);
	gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
}


#[wasm_bindgen(start)]
pub fn web_main() -> Result<(), JsValue> {

}


pub fn compile_shader(
	gl: &WebGl2RenderingContext,
	shader_type: u32,
	source: &str,
) -> Result<WebGlShader, String> {
	let shader = gl
		.create_shader(shader_type)
		.ok_or_else(|| String::from("Unable to create shader object"))?;
	gl.shader_source(&shader, source);
	gl.compile_shader(&shader);

	if gl
		.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
		.as_bool()
		.unwrap_or(false)
	{
		Ok(shader)
	} else {
		Err(gl
			.get_shader_info_log(&shader)
			.unwrap_or_else(|| "Unknown error creating shader".into()))
	}
}


pub fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
	gl: &WebGl2RenderingContext,
	shaders: T,
) -> Result<WebGlProgram, String> {
	let program = gl
		.create_program()
		.ok_or_else(|| String::from("Unable to create shader object"))?;
	for shader in shaders {
		gl.attach_shader(&program, shader)
	}
	gl.link_program(&program);

	if gl
		.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
		.as_bool()
		.unwrap_or(false)
	{
		Ok(program)
	} else {
		Err(gl
			.get_program_info_log(&program)
			.unwrap_or_else(|| "Unknown error creating program object".into()))
	}
}


fn init_gl() -> Result<(), JsValue> {
	let document = web_sys::window().unwrap().document().unwrap();
	let canvas = document.create_element("canvas")?.dyn_into::<web_sys::HtmlCanvasElement>()?;

	//let canvas = document.get_element_by_id("canvas").unwrap();
	//let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

	let context = canvas
			.get_context("webgl2")?
			.unwrap()
			.dyn_into::<WebGl2RenderingContext>()?;
}


pub struct GLWeb {
	canvas: HtmlCanvasElement,
	gl: WebGl2RenderingContext,
}


impl GLWeb {

}


impl<'a> GLSys<'a> for GLWeb {
	fn new() -> Result<Self, &'a str> {
		log(format!("Initializing WebGL").as_ref());

		let (_canvas, gl) = init_gl()?;

		let vert_shader = compile_shader(&gl, WebGl2RenderingContext::VERTEX_SHADER, VERTEX_SHADER_0)?;
		let frag_shader = compile_shader(&gl, WebGl2RenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER_0)?;
		let program = link_program(&gl, [vert_shader, frag_shader].iter())?;
		gl.use_program(Some(&program));

		let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
		let memory_buffer = wasm_bindgen::memory()
			.dyn_into::<WebAssembly::Memory>()?
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

		/* gl.clear_color(0.0, 0.0, 0.0, 1.0);
		gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

		gl.draw_arrays(
			WebGl2RenderingContext::TRIANGLES,
			0,
			(vertices.len() / 3) as i32,
		); */

		Ok(())
	}


	fn start_loop(&self) {
		let f = Rc::new(RefCell::new(None));
		let g = f.clone();

		let mut frame: usize = 0;

		*g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
			// let _ = f.borrow_mut().take(); // kill
			draw_frame(&gl, &frame);
			request_animation_frame(f.borrow().as_ref().unwrap());

			frame += 1;
		}) as Box<FnMut()>));

		request_animation_frame(g.borrow().as_ref().unwrap());
	}
}
