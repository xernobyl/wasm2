const SIZE_IN_MB: usize = 16;


struct StaticGeometry {
	gl: &WebGl2RenderingContext,
}


pub impl StaticGeometry {
	pub fn new(gl: &WebGl2RenderingContext) -> Self {
		StaticGeometry {
			gl: gl,
		}
	}

	pub fn add_vertices(&self, data: X) {

	}

	pub fn bindVertices(&self) {
		self.gl.bindBuffer(gl.ARRAY_BUFFER, self.vbo)
	}

	pub fn bindElements(&self) {
		self.gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.ebo)
	}
}


impl Drop for StaticGeometry {

}




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