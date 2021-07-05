/*
Store a cube as a FAN of visible faces, do fancy stuff on vertex shader.
Might be "optimal", according to people on the internet.
*/

use std::rc::Rc;

use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

type Gl = WebGl2RenderingContext;

use crate::utils::as_u8_slice;

pub struct HalfCube {
    gl: Rc<Gl>,
    vao: WebGlVertexArrayObject,
    index_buffer: WebGlBuffer,
    vertex_buffer: WebGlBuffer,
    mvp_buffer: WebGlBuffer,
    position_buffer: WebGlBuffer,
}

impl HalfCube {
    pub fn new(gl: Rc<Gl>) -> Self {
        const INDEX_BUFFER: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 1];
        const VERTEX_BUFFER: [i8; 7 * 3] = [
            1, 1, 1, -1, 1, 1, -1, -1, 1, 1, -1, 1, 1, -1, -1, 1, 1, -1, -1, 1, -1,
        ];

        let vao = gl.create_vertex_array().expect("Error creating VAO.");
        gl.bind_vertex_array(Some(&vao));

        let vertex_buffer = gl.create_buffer().expect("Error creating ARRAY_BUFFER.");
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_u8_array(
            Gl::ARRAY_BUFFER,
            as_u8_slice(&VERTEX_BUFFER),
            Gl::STATIC_DRAW,
        );

        let index_buffer = gl
            .create_buffer()
            .expect("Error creating ELEMENT_ARRAY_BUFFER.");
        gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        gl.buffer_data_with_u8_array(Gl::ELEMENT_ARRAY_BUFFER, &INDEX_BUFFER, Gl::STATIC_DRAW);

        gl.vertex_attrib_pointer_with_i32(0, 3, Gl::BYTE, false, 3, 0);
        gl.enable_vertex_attrib_array(0);

        // if enabled instancing? too unlikely that only a single cube is drawn...
        let mvp_buffer = gl
            .create_buffer()
            .expect("Error creating ELEMENT_ARRAY_BUFFER.");
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&mvp_buffer));
        // gl.buffer_data_with_u8_array(Gl::ARRAY_BUFFER, data_size, Gl::DYNAMIC_DRAW);

        let mvp_index = 1;

        gl.vertex_attrib_pointer_with_i32(mvp_index + 0, 4, Gl::FLOAT, false, 64, 0);
        gl.vertex_attrib_pointer_with_i32(mvp_index + 1, 4, Gl::FLOAT, false, 64, 4 * 4);
        gl.vertex_attrib_pointer_with_i32(mvp_index + 2, 4, Gl::FLOAT, false, 64, 8 * 4);
        gl.vertex_attrib_pointer_with_i32(mvp_index + 3, 4, Gl::FLOAT, false, 64, 12 * 4);

        gl.enable_vertex_attrib_array(mvp_index + 0);
        gl.enable_vertex_attrib_array(mvp_index + 1);
        gl.enable_vertex_attrib_array(mvp_index + 2);
        gl.enable_vertex_attrib_array(mvp_index + 3);

        gl.vertex_attrib_divisor(mvp_index + 0, 1);
        gl.vertex_attrib_divisor(mvp_index + 1, 1);
        gl.vertex_attrib_divisor(mvp_index + 2, 1);
        gl.vertex_attrib_divisor(mvp_index + 3, 1);

        let position_buffer = gl
            .create_buffer()
            .expect("Error creating ELEMENT_ARRAY_BUFFER.");
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&position_buffer));
        gl.vertex_attrib_pointer_with_i32(5, 3, Gl::FLOAT, false, 3 * 4, 0);
        gl.enable_vertex_attrib_array(5);
        gl.vertex_attrib_divisor(5, 1);

        Self {
            gl,
            vao,
            index_buffer,
            vertex_buffer,
            mvp_buffer,
            position_buffer,
        }
    }

    pub fn draw(&self) {
        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl
            .draw_elements_with_i32(Gl::TRIANGLE_FAN, 8, Gl::UNSIGNED_BYTE, 0);
    }

    pub fn draw_instanced(&self, count: i32) {
        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl
            .draw_elements_instanced_with_i32(Gl::TRIANGLE_FAN, 8, Gl::UNSIGNED_BYTE, 0, count);
    }

    pub fn update_mvp(&self, matrices: &[f32]) {
        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl
            .bind_buffer(Gl::ARRAY_BUFFER, Some(&self.mvp_buffer));
        self.gl.buffer_data_with_u8_array(
            Gl::ARRAY_BUFFER,
            as_u8_slice(matrices),
            Gl::DYNAMIC_DRAW,
        );
    }

    pub fn update_positions(&self, positions: &[f32]) {
        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl
            .bind_buffer(Gl::ARRAY_BUFFER, Some(&self.position_buffer));
        self.gl.buffer_data_with_u8_array(
            Gl::ARRAY_BUFFER,
            as_u8_slice(positions),
            Gl::DYNAMIC_DRAW,
        );
    }
}

impl Drop for HalfCube {
    fn drop(&mut self) {
        self.gl.delete_vertex_array(Some(&self.vao));
        self.gl.delete_buffer(Some(&self.vertex_buffer));
        self.gl.delete_buffer(Some(&self.index_buffer));
        self.gl.delete_buffer(Some(&self.mvp_buffer));
        self.gl.delete_buffer(Some(&self.position_buffer));
    }
}
