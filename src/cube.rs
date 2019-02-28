// cube.rs

use crate::static_geometry::StaticGeometry;
use web_sys::{WebGlVertexArrayObject, WebGl2RenderingContext};
type GL = WebGl2RenderingContext;

pub struct Cube {
  vao: WebGlVertexArrayObject,
  vertex_offet: usize,
  element_offset: usize,
}


impl Cube {
  pub fn new(gl: &GL, static_geometry: &mut StaticGeometry) -> Self {
    const VERTEX_BUFFER: [i8; 192] =
      [1, 1, 1, 0, 0, 1, 1, 1, -1, 1,1 ,0,0,1,0,1,-1,-1,1,0,0,1,0,0,1,-1,1,0,0,1,1,0,1,1,1,1,0,0,0,1,1,-1,1,1,0,0,0,0,1,-1,-1,1,0,0,1,0,1,1,-1,1,0,0,1,1,1,1,1,0,1,0,1,0,1,1,-1,0,1,0,1,1,-1,1,-1,0,1,0,0,1,-1,1,1,0,1,0,0,0,-1,1,1,-1,0,0,1,1,-1,1,-1,-1,0,0,0,1,-1,-1,-1,-1,0,0,0,0,-1,-1,1,-1,0,0,1,0,-1,-1,-1,0,-1,0,0,0,1,-1,-1,0,-1,0,1,0,1,-1,1,0,-1,0,1,1,-1,-1,1,0,-1,0,0,1,1,-1,-1,0,0,-1,0,0,-1,-1,-1,0,0,-1,1,0,-1,1,-1,0,0,-1,1,1,1,1,-1,0,0,-1,0,1];

    const INDEX_BUFFER: [u8; 60] =
      [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
      // outlines
      0, 1, 0, 3, 0, 7, 1, 2, 2, 5, 5, 6, 6, 7, 10, 11, 13, 14, 15, 16, 20, 21, 22, 23];

    let vao = gl.create_vertex_array().unwrap();
		gl.bind_vertex_array(Some(&vao));

    let vertex_offet = static_geometry.add_vertices(gl, unsafe { std::slice::from_raw_parts_mut(VERTEX_BUFFER.as_mut_ptr() as *mut u8, VERTEX_BUFFER.len()) });
    let element_offset = static_geometry.add_elements(gl, &mut INDEX_BUFFER);

		gl.vertex_attrib_pointer_with_i32(0, 3, GL::BYTE, false, 8, vertex_offet as i32 + 0);
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_with_i32(1, 3, GL::BYTE, false, 8, vertex_offet as i32 + 3);
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_with_i32(2, 2, GL::BYTE, false, 8, vertex_offet as i32 + 6);
    gl.enable_vertex_attrib_array(2);

    Cube {
      vao,
      vertex_offet,
      element_offset,
    }
  }

  pub fn draw(&self, gl: &GL) {
		gl.bind_vertex_array(Some(&self.vao));
		gl.draw_elements_with_i32(GL::TRIANGLES, 36, GL::UNSIGNED_BYTE, self.element_offset as i32);
  }

  pub fn draw_outlines(&self, gl: &GL) {
    gl.bind_vertex_array(Some(&self.vao));
    gl.draw_elements_with_i32(GL::LINES, 24, GL::UNSIGNED_BYTE, self.element_offset as i32 + 36);
  }
}
