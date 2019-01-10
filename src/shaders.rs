// shaders.rs

const VERTEX_SHADER_0 = r#"
	attribute vec4 position;
	void main() {
		gl_Position = position;
	}
"#;

const FRAGMENT_SHADER_0 = r#"
	void main() {
		gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
	}
"#;

pub struct shaders {
	gl: &WebGl2RenderingContext,
};

pub impl shaders {
	pub fn init(gl: &WebGl2RenderingContext) -> Self {
		Self {
			gl: gl,
		}
	}

	fn compile_shader(&self,
		gl: &WebGl2RenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
		let shader = gl.create_shader(shader_type)?;

		gl.shader_source(&shader, source)?;
		gl.compile_shader(&shader)?;

		if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
			Ok(shader)
		} else {
			Err(self.gl
				.get_shader_info_log(&shader)
				.unwrap_or_else(|| "Unknown error creating shader".into()))
		}
	}

	pub fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
		&self, gl: &WebGl2RenderingContext, shaders: T) -> Result<WebGlProgram, String> {
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

}
