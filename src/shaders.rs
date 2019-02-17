use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};


struct Shader {
	program: WebGlProgram,
}


pub struct ShaderManager {
	shaders: Vec<Shader>,
}


impl ShaderManager {
	pub fn new(/* gl: &WebGl2RenderingContext */) -> ShaderManager {
		ShaderManager {
			shaders: Vec::new(),
		}
	}


	fn build_shader(
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
		.as_bool().unwrap_or(false)
		{
			Ok(shader)
		} else {
			Err(gl
			.get_shader_info_log(&shader)
			.unwrap_or_else(|| "Unknown error creating shader".into()))
		}
	}


	pub fn create_shader(
		&mut self,
		gl: &WebGl2RenderingContext,
		vertex_source: &str,
		fragment_source: &str
	) -> Result<usize, String> {
		let vertex_shader = Self::build_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vertex_source)?;
		let fragment_shader = Self::build_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, fragment_source)?;

		let program = gl
		.create_program()
		.ok_or_else(|| String::from("Unable to create shader object"))?;

		gl.attach_shader(&program, &vertex_shader);
		gl.attach_shader(&program, &fragment_shader);
		gl.link_program(&program);

		if gl
			.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
			.as_bool()
			.unwrap_or(false)
		{
			self.shaders.push(Shader {
				program,
			});

			Ok(self.shaders.len())
		} else {
			Err(gl
				.get_program_info_log(&program)
				.unwrap_or_else(|| "Unknown error creating program object".into()))
		}
	}


	pub fn bind_shader(&self, gl: &WebGl2RenderingContext, id: usize) {
		gl.use_program(Some(&self.shaders[id].program));
	}
}
