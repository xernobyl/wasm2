/*
Lines!
*/

/*
TODO: try drawing giant strip to take advantage of caching
current method probably takes

Maybe render lines (2 verts) to a buffer using
transform feedback, and reusing it as a strip?
*/

use std::{borrow::Borrow, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

type Gl = WebGl2RenderingContext;

use crate::utils::as_u8_slice;

pub struct Line2DStrip {
    gl: Rc<Gl>,
    vao: WebGlVertexArrayObject,
    position_buffer: WebGlBuffer,
}

impl Line2DStrip {
    pub fn new(gl: &Rc<Gl>) -> Self {
        log!("new line strip");

        let vao = gl.create_vertex_array().expect("Error creating VAO.");
        gl.bind_vertex_array(Some(&vao));

        let position_buffer = gl
            .create_buffer()
            .expect("Error creating ELEMENT_ARRAY_BUFFER.");
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&position_buffer));

        let stride = 4 * 3 * 1;

        gl.vertex_attrib_pointer_with_i32(0, 3, Gl::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_divisor(0, 1);

        gl.vertex_attrib_pointer_with_i32(1, 3, Gl::FLOAT, false, stride, 12);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_divisor(1, 1);

        gl.vertex_attrib_pointer_with_i32(2, 3, Gl::FLOAT, false, stride, 24);
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_divisor(2, 1);

        gl.vertex_attrib_pointer_with_i32(3, 3, Gl::FLOAT, false, stride, 36);
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_divisor(3, 1);

        Self {
            gl: gl.clone(),
            vao,
            position_buffer,
        }
    }

    pub fn draw(&self, gl: &Gl, count: i32) {
        gl.bind_vertex_array(Some(&self.vao));
        gl.draw_arrays_instanced(Gl::TRIANGLE_STRIP, 0, 4, count);
    }

    pub fn update_points(&self, gl: &Gl, points: &[f32]) {
        /* format for points is x, y, width */

        gl.bind_vertex_array(Some(&self.vao));
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.position_buffer));
        gl.buffer_data_with_u8_array(Gl::ARRAY_BUFFER, as_u8_slice(points), Gl::DYNAMIC_DRAW);
    }
}

impl Drop for Line2DStrip {
    fn drop(&mut self) {
        log!("drop line strip");
        self.gl.delete_vertex_array(Some(&self.vao));
        self.gl.delete_buffer(Some(&self.position_buffer));
    }
}
