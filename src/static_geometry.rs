use web_sys::{WebGlBuffer, WebGl2RenderingContext};
use wasm_bindgen::prelude::*;

type GL = WebGl2RenderingContext;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console, js_name = log)]
	fn log(s: &str);
}


pub struct StaticGeometry {
	vbo: WebGlBuffer,
	vertex_buffer_size: usize,
	vertex_last_offset: usize,

	ebo: WebGlBuffer,
	element_buffer_size: usize,
	element_last_offset: usize,
}


impl StaticGeometry {
	pub fn new(gl: &GL, vertex_buffer_size: usize, element_buffer_size: usize) -> Result<Self, String> {
		log("Initializing StaticGeometry:");
		let vbo = gl.create_buffer().ok_or("failed to create buffer")?;
		gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
		gl.buffer_data_with_i32(
			GL::ARRAY_BUFFER,
			vertex_buffer_size as i32,
			GL::STATIC_DRAW
		);

		let ebo = gl.create_buffer().ok_or("failed to create buffer")?;
		gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ebo));
		gl.buffer_data_with_i32(
			GL::ELEMENT_ARRAY_BUFFER,
			element_buffer_size as i32,
			GL::STATIC_DRAW
		);

		log("All done.");

		Ok(StaticGeometry {
			vbo,
			vertex_last_offset: 0,
			vertex_buffer_size,

			ebo,
			element_last_offset: 0,
			element_buffer_size,
		})
	}

	pub fn add_vertices(&mut self, gl: &GL, data: &mut [u8]) -> usize {
		gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));

		// TODO: use correct size, confirm this works
		let size = 6;
		let padding = !(0xffff_ffff << size);

		if self.vertex_last_offset & padding != 0 {
			self.vertex_last_offset += !(self.vertex_last_offset & padding) & padding;
		};

		gl.buffer_sub_data_with_i32_and_u8_array(
			GL::ARRAY_BUFFER,
			self.vertex_last_offset as i32,
			data
		);

		let offset = self.vertex_last_offset;
		self.vertex_last_offset += data.len();

		if self.vertex_buffer_size < self.vertex_last_offset {
			panic!("VERTEX BUFFER NEEDS MORE MEMORY");
		}

		offset
	}

	pub fn add_elements(&mut self, gl: &GL, data: &mut [u8]) -> usize {
		gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.ebo));

		// TODO: use correct size, confirm this works
		let size = 6;
		let padding = !(0xffff_ffff << size);

		if self.element_last_offset & padding != 0 {
			self.element_last_offset += !(self.element_last_offset & padding) & padding;
		};

		gl.buffer_sub_data_with_i32_and_u8_array(
			GL::ELEMENT_ARRAY_BUFFER,
			self.element_last_offset as i32,
			data
		);

		let offset = self.element_last_offset;
		self.element_last_offset += data.len();

		if self.element_buffer_size < self.element_last_offset {
			panic!("ELEMENT BUFFER NEEDS MORE MEMORY");
		}

		offset
	}

	pub fn bind_vertices(&self, gl: &GL) {
		gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));
	}

	pub fn bind_elements(&self, gl: &GL) {
		gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.ebo));
	}
}


impl Drop for StaticGeometry {
	fn drop(&mut self) {
		// TODO: this
	}
}
