use crate::app::{App, Programs};
use crate::fullscreen_buffers::{self, ScreenBuffers};
use crate::scene::Scene;
use crate::{fast_rand, line_2d_strip, utils};
use glam::{Mat4, Vec3};
use serde::Serialize;
use std::{panic};
use std::{rc::Rc, cell::RefCell};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};

type Gl = WebGl2RenderingContext;

pub struct Scene1 {

}


impl Scene for Scene1 {
  fn new() -> Self {
    Self {

    }
  }

  fn init(&mut self) {

  }


  fn on_frame(&self, app: &App) {
    /*
    Use infinite inverted depth buffer because of the better precision
    */

    let gl = &app.context;
    let mut rng = fast_rand::FastRand::new(3464357);

    // log!0"Frame: {}\nTimestamp: {}", self.current_frame, self.current_timestamp);

    app.fullscreen_buffers.bind(&gl);
    gl.clear_color(rng.urand(), rng.urand(), rng.urand(), rng.urand());
    gl.clear(Gl::DEPTH_BUFFER_BIT | Gl::COLOR_BUFFER_BIT);

    let camera_position = Vec3::new(15.0 * f32::cos(app.current_timestamp as f32 / 2000.0), 10.0 * f32::sin(app.current_timestamp as f32 / 2000.0), 15.0 * f32::sin(app.current_timestamp as f32 / 2000.0));
    let projection = Mat4::perspective_infinite_reverse_rh(std::f32::consts::PI / 2.0, app.aspect_ratio, 0.1);
    let view = Mat4::look_at_rh(camera_position, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

    let view_projection = projection * view;

    let mut view_projection_array = Vec::new();
    // let mut position_array = Vec::new();

    for _ in 0..128 {
      let scale_factor = rng.urand() * 0.75 + 0.25;

      let cube_pos = Vec3::new(5.0 * rng.rand(), 5.0 * rng.rand(), 5.0 * rng.rand());
      let mv = Mat4::from_translation(cube_pos);

      // camera aligned cubes... save this for later, probably do it in shader
      let mut dir = camera_position - cube_pos;
      dir.x = if dir.x >= 0.0 { 1.0 } else { -1.0 };
      dir.y = if dir.y >= 0.0 { 1.0 } else { -1.0 };
      dir.z = if dir.z >= 0.0 { 1.0 } else { -1.0 };
      let scale = Mat4::from_scale(dir);

      view_projection_array.push(view_projection * mv * scale);
      // position_array.push(pos);
    };

    // self.cube.update_mvp(utils::as_f32_slice(view_projection_array.as_slice(), 4 * 4));
    // self.cube.update_positions(&gl, utils::as_f32_slice(position_array.as_slice(), 3));

    gl.use_program(app.programs[Programs::Cube as usize].as_ref());
    //let location = gl.get_uniform_location(self.programs[Programs::Cube as usize].as_ref().unwrap(), "camera");

    //gl.uniform_matrix4fv_with_f32_array(location.as_ref(), false, &m.to_cols_array());
    //let location = gl.get_uniform_location(self.programs[Programs::Cube as usize].as_ref().unwrap(), "camera_position");
    //gl.uniform3f(location.as_ref(), camera_position.x, camera_position.y, camera_position.z);

    gl.depth_func(Gl::GREATER);
    gl.clear_depth(0.0);
    gl.enable(Gl::DEPTH_TEST);
    // self.cube.draw_instanced(view_projection_array.len() as i32);
    gl.disable(Gl::DEPTH_TEST);

    let mut lines = Vec::new();

    for i in 0..500 {
      lines.push(f32::sin(i as f32 / 500.0 * std::f32::consts::TAU + app.current_timestamp as f32 / 2000.0));
      lines.push(f32::cos(i as f32 / 500.0 * std::f32::consts::TAU + app.current_timestamp as f32 / 2000.0));
      lines.push(rng.urand() * 0.05);

      // lines.push(0.05);
    };

    let mut_lines = line_2d_strip::Line2DStrip::new(&gl);
    mut_lines.update_points(lines.as_slice());

    gl.use_program(app.programs[Programs::Line2DStrip as usize].as_ref());
    mut_lines.draw(500 - 3);

    // screen pass
    gl.bind_framebuffer(Gl::FRAMEBUFFER, None);
    gl.viewport(0, 0, app.width as i32, app.height as i32);

    gl.use_program(app.programs[Programs::Screen as usize].as_ref());
    let location = gl.get_uniform_location(app.programs[Programs::Cube as usize].as_ref().unwrap(), "color_texture");
    gl.active_texture(Gl::TEXTURE0);
    gl.bind_texture(Gl::TEXTURE_2D, Some(&app.fullscreen_buffers.color_texture));
    gl.uniform1i(location.as_ref(), 0);
    utils::fullscreen_quad(&gl);
  }
}


impl Drop for Scene1 {
  fn drop(&mut self) {
  }
}
