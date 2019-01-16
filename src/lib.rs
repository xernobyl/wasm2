use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};

pub mod gl_web;
pub mod gl_sys;

use wasm_bindgen::prelude::*;

use self::gl_web::{GLWeb, gl_init};
use self::gl_sys::GLSys;


#[wasm_bindgen(start)]
pub fn web_main() -> Result<(), JsValue> {
	let gl = GLWeb::new()?;
	gl.start_loop();
	// Ok(())

	//////////////////

	let (canvas, context) = gl_init()?;

	let vert_shader = compile_shader(
			&context,
			WebGl2RenderingContext::VERTEX_SHADER,
			r#"
			attribute vec4 position;
			void main() {
					gl_Position = position;
			}
	"#,
	)?;
	let frag_shader = compile_shader(
			&context,
			WebGl2RenderingContext::FRAGMENT_SHADER,
			r#"
			void main() {
					gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
			}
	"#,
	)?;
	let program = link_program(&context, [vert_shader, frag_shader].iter())?;
	context.use_program(Some(&program));

	let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
	let memory_buffer = wasm_bindgen::memory()
			.dyn_into::<WebAssembly::Memory>()?
			.buffer();
	let vertices_location = vertices.as_ptr() as u32 / 4;
	let vert_array = js_sys::Float32Array::new(&memory_buffer)
			.subarray(vertices_location, vertices_location + vertices.len() as u32);

	let buffer = context.create_buffer().ok_or("failed to create buffer")?;
	context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
	context.buffer_data_with_array_buffer_view(
			WebGl2RenderingContext::ARRAY_BUFFER,
			&vert_array,
			WebGl2RenderingContext::STATIC_DRAW,
	);
	context.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
	context.enable_vertex_attrib_array(0);

	context.clear_color(0.0, 0.0, 0.0, 1.0);
	context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

	context.draw_arrays(
			WebGl2RenderingContext::TRIANGLES,
			0,
			(vertices.len() / 3) as i32,
	);
	Ok(())
}


pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}

pub fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
    context: &WebGl2RenderingContext,
    shaders: T,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    for shader in shaders {
        context.attach_shader(&program, shader)
    }
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program object".into()))
    }
}
