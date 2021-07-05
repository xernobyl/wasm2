/*
Shaders
*/

enum Programs {
    Screen,
    Cube,
    DepthMaxMin0,
    DepthMaxMin1,
    NPrograms,
}

fn setup_shaders(&mut self) -> Result<(), JsValue> {
    let gl = &self.context;

    let vert_shader =
        Self::compile_shader(gl, Gl::VERTEX_SHADER, include_str!("glsl/screen.vert"))?;
    let frag_shader =
        Self::compile_shader(gl, Gl::FRAGMENT_SHADER, include_str!("glsl/screen.frag"))?;
    self.programs[Programs::Screen as usize] =
        Some(Self::link_program(gl, &vert_shader, &frag_shader)?);
    gl.delete_shader(Some(&frag_shader));
    gl.delete_shader(Some(&vert_shader));

    let vert_shader =
        Self::compile_shader(gl, Gl::VERTEX_SHADER, include_str!("glsl/cube_basic.vert"))?;
    let frag_shader = Self::compile_shader(
        gl,
        Gl::FRAGMENT_SHADER,
        include_str!("glsl/cube_basic.frag"),
    )?;
    self.programs[Programs::Cube as usize] =
        Some(Self::link_program(gl, &vert_shader, &frag_shader)?);
    gl.delete_shader(Some(&frag_shader));
    gl.delete_shader(Some(&vert_shader));

    let vert_shader =
        Self::compile_shader(gl, Gl::VERTEX_SHADER, include_str!("glsl/max_min.vert"))?;
    let frag_shader = Self::compile_shader(
        gl,
        Gl::FRAGMENT_SHADER,
        include_str!("glsl/depth_max_min.frag"),
    )?;
    self.programs[Programs::DepthMaxMin0 as usize] =
        Some(Self::link_program(gl, &vert_shader, &frag_shader)?);
    gl.delete_shader(Some(&frag_shader));
    gl.delete_shader(Some(&vert_shader));

    let vert_shader =
        Self::compile_shader(gl, Gl::VERTEX_SHADER, include_str!("glsl/max_min.vert"))?;
    let frag_shader = Self::compile_shader(
        gl,
        Gl::FRAGMENT_SHADER,
        include_str!("glsl/max_min_max_min.frag"),
    )?;
    self.programs[Programs::DepthMaxMin1 as usize] =
        Some(Self::link_program(gl, &vert_shader, &frag_shader)?);
    gl.delete_shader(Some(&frag_shader));
    gl.delete_shader(Some(&vert_shader));

    Ok(())
}

fn compile_shader(context: &Gl, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error getting shader info log")))
    }
}

fn link_program(
    context: &Gl,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if !context
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    } else {
        Ok(program)
    }
}
