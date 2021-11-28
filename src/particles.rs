/*
Particles!
*/


use std::{borrow::Borrow, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

type Gl = WebGl2RenderingContext;

use crate::utils::as_u8_slice;

pub struct Particles<'a> {
    gl: &'a Gl,
    vao: WebGlVertexArrayObject,
    position_buffer: WebGlBuffer,
}

impl <'a> Particles<'a> {
    pub fn new(gl: &'a Gl) -> Self {
        log!("new particles");

        let vao = gl.create_vertex_array().expect("Error creating VAO.");
        gl.bind_vertex_array(Some(&vao));

        let position_buffer = gl
            .create_buffer()
            .expect("Error creating ELEMENT_ARRAY_BUFFER.");
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&position_buffer));

        let stride = 4 * 3 * 1;

        gl.vertex_attrib_pointer_with_i32(0, 3, Gl::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);

        Self {
            gl,
            vao,
            position_buffer,
        }
    }

    pub fn draw(&self, gl: &Gl, count: i32) {
        /*
        6 points: draw an hexagon, should be a bit more eficient than a square
          2---4
         /|\  |\
        0 | \ | 5
         \|  \|/
          1---3
        */

        gl.bind_vertex_array(Some(&self.vao));
        gl.draw_arrays_instanced(Gl::TRIANGLE_STRIP, 0, 6, count);
    }

    pub fn update_points(&self, gl: &Gl, points: &[f32]) {
        /* format for points is x, y, width */

        gl.bind_vertex_array(Some(&self.vao));
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.position_buffer));
        gl.buffer_data_with_u8_array(Gl::ARRAY_BUFFER, as_u8_slice(points), Gl::DYNAMIC_DRAW);
    }
}

impl <'a> Drop for Particles<'a> {
    fn drop(&mut self) {
        log!("drop particles");
        let gl: &Gl = self.gl.borrow();

        gl.delete_vertex_array(Some(&self.vao));
        gl.delete_buffer(Some(&self.position_buffer));
    }
}
