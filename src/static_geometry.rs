use web_sys::{WebGlBuffer, WebGl2RenderingContext};

const SIZE_IN_MB: usize = 16;


pub struct StaticGeometry {
	vbo: WebGlBuffer,
	ebo: WebGlBuffer,
	last_offset: usize,
	buffer_size: usize,
}


impl StaticGeometry {
	pub fn new(gl: &WebGl2RenderingContext, buffer_size: usize) -> Result<Self, String> {
		let vbo = gl.create_buffer().ok_or("failed to create buffer")?;
		let ebo = gl.create_buffer().ok_or("failed to create buffer")?;
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
		gl.buffer_data_with_i32(
			WebGl2RenderingContext::ARRAY_BUFFER,
			buffer_size as i32,
			WebGl2RenderingContext::STATIC_DRAW
		);
		let last_offset = 0;

		Ok(StaticGeometry {
			vbo,
			ebo,
			last_offset,
			buffer_size,
		})
	}

	pub fn add_vertices(&self, gl: &WebGl2RenderingContext, data: X) -> usize {
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo));

		// TODO: use correct size
		let size = 64;
		let padding = !(0xffffffff << size);
		self.last_offset += !(self.last_offset & padding) & padding;

		gl.buffer_sub_data_with_i32(
			WebGl2RenderingContext::ARRAY_BUFFER,
			self.last_offset as i32,
			data
		);

		let offset = self.last_offset;
		self.last_offset += data.byteLength;

		if self.buffer_size < self.last_offset {
			panic!("BUFFER NEEDS MORE MEMORY");
		}

		offset
	}

	pub fn bindVertices(&self, gl: &WebGl2RenderingContext) {
		gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo));
	}

	pub fn bindElements(&self, gl: &WebGl2RenderingContext) {
		gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.ebo));
	}
}

// TODO: this
/*
impl Drop for StaticGeometry {
}
*/



/*
class StaticGeometry {
	static addVertices(data) {
		const SIZE_IN_MB = 16

		if (this.vbo === undefined) {
			this.vbo = gl.createBuffer()
			gl.bindBuffer(gl.ARRAY_BUFFER, this.vbo)
			gl.bufferData(gl.ARRAY_BUFFER, SIZE_IN_MB * 1048576, gl.STATIC_DRAW)
			this.lastOffset = 0
		}
		else {
			gl.bindBuffer(gl.ARRAY_BUFFER, this.vbo)

			console.log('before alignment', this.lastOffset)
			const alignment = data.byteLength / data.length
			this.lastOffset = Math.ceil(this.lastOffset / alignment) * alignment
			console.log('after alignment', this.lastOffset)
		}

		gl.bufferSubData(gl.ARRAY_BUFFER, this.lastOffset, data)

		const offset = this.lastOffset
		this.lastOffset += data.byteLength

		if (SIZE_IN_MB * 1048576 < this.lastOffset)
			console.log('WARNING: INCREASE MAX SIZE TO ' + this.lastOffset)

		return offset
	}

	static bindVertices() {
		gl.bindBuffer(gl.ARRAY_BUFFER, this.vbo)
	}

	static addElements(data) {
		const ELEMENT_SIZE_IN_MB = 2

		if (this.ebo === undefined) {
			this.ebo = gl.createBuffer()
			gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.ebo)
			gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, ELEMENT_SIZE_IN_MB * 1048576, gl.STATIC_DRAW)
			this.lastElementOffset = 0
		}
		else {
			gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.ebo)

			console.log('before alignment', this.lastElementOffset)
			const alignment = data.byteLength / data.length
			this.lastElementOffset = Math.ceil(this.lastElementOffset / alignment) * alignment
			console.log('after alignment', this.lastElementOffset)
		}

		gl.bufferSubData(gl.ELEMENT_ARRAY_BUFFER, this.lastElementOffset, data)

		const offset = this.lastElementOffset
		this.lastElementOffset += data.byteLength

		if (ELEMENT_SIZE_IN_MB * 1048576 < this.lastElementOffset)
			console.log('WARNING: INCREASE MAX SIZE TO ' + this.lastElementOffset)

		return offset
	}

	static bindElements() {
		gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.ebo)
	}
}
*/