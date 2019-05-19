// quad.rs

use crate::static_geometry::StaticGeometry;
use web_sys::{WebGlVertexArrayObject, WebGl2RenderingContext};
type GL = WebGl2RenderingContext;

pub struct Quad {
  vao: WebGlVertexArrayObject,
  vertex_offet: usize,
}


impl Quad {
  pub fn new(gl: &GL, static_geometry: &mut StaticGeometry) -> Self {
    const VERTEX_BUFFER: [i8; 6] =
      [-100, -2, 100, -2, 0, 10];

    let vao = gl.create_vertex_array().unwrap();
		gl.bind_vertex_array(Some(&vao));

    let vertex_offet = static_geometry.add_vertices(gl, unsafe { std::slice::from_raw_parts_mut(VERTEX_BUFFER.as_mut_ptr() as *mut u8, VERTEX_BUFFER.len()) });
    gl.vertex_attrib_pointer_with_i32(0, 2, GL::BYTE, false, 2, vertex_offet as i32);
    gl.enable_vertex_attrib_array(0);

    Quad {
      vao,
      vertex_offet,
    }
  }

  pub fn draw(&self, gl: &GL) {
		gl.bind_vertex_array(Some(&self.vao));
		gl.draw_arrays(GL::TRIANGLES, self.vertex_offet as i32, 6);
  }
}
