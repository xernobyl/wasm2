// cube.rs

static vertex_buffer: [i8; _] = [1,1,1,0,0,1,1,1,-1,1,1,0,0,1,0,1,-1,-1,1,0,0,1,0,0,1,-1,1,0,0,1,1,0,1,1,1,1,0,0,0,1,1,-1,1,1,0,0,0,0,1,-1,-1,1,0,0,1,0,1,1,-1,1,0,0,1,1,1,1,1,0,1,0,1,0,1,1,-1,0,1,0,1,1,-1,1,-1,0,1,0,0,1,-1,1,1,0,1,0,0,0,-1,1,1,-1,0,0,1,1,-1,1,-1,-1,0,0,0,1,-1,-1,-1,-1,0,0,0,0,-1,-1,1,-1,0,0,1,0,-1,-1,-1,0,-1,0,0,0,1,-1,-1,0,-1,0,1,0,1,-1,1,0,-1,0,1,1,-1,-1,1,0,-1,0,0,1,1,-1,-1,0,0,-1,0,0,-1,-1,-1,0,0,-1,1,0,-1,1,-1,0,0,-1,1,1,1,1,-1,0,0,-1,0,1];
static indices_buffer: [u8; _] = [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
      // outlines
      0, 1, 0, 3, 0, 7, 1, 2, 2, 5, 5, 6, 6, 7, 10, 11, 13, 14, 15, 16, 20, 21, 22, 23];

struct Cube {
	vao: u32,
	element_offset: usize,
}


pub impl Cube {

}

/*
class Cube {
  static init() {
    const vertexBuffer = new Int8Array([
      1,1,1,0,0,1,1,1,-1,1,1,0,0,1,0,1,-1,-1,1,0,0,1,0,0,1,-1,1,0,0,1,1,0,1,1,1,1,0,0,0,1,1,-1,1,1,0,0,0,0,1,-1,-1,1,0,0,1,0,1,1,-1,1,0,0,1,1,1,1,1,0,1,0,1,0,1,1,-1,0,1,0,1,1,-1,1,-1,0,1,0,0,1,-1,1,1,0,1,0,0,0,-1,1,1,-1,0,0,1,1,-1,1,-1,-1,0,0,0,1,-1,-1,-1,-1,0,0,0,0,-1,-1,1,-1,0,0,1,0,-1,-1,-1,0,-1,0,0,0,1,-1,-1,0,-1,0,1,0,1,-1,1,0,-1,0,1,1,-1,-1,1,0,-1,0,0,1,1,-1,-1,0,0,-1,0,0,-1,-1,-1,0,0,-1,1,0,-1,1,-1,0,0,-1,1,1,1,1,-1,0,0,-1,0,1])
    const indicesBuffer = new Uint8Array([
      0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
      // outlines
      0, 1, 0, 3, 0, 7, 1, 2, 2, 5, 5, 6, 6, 7, 10, 11, 13, 14, 15, 16, 20, 21, 22, 23
    ])

		this.vao = gl.createVertexArray()
		gl.bindVertexArray(this.vao)

    let vertexOffet = StaticGeometry.addVertices(vertexBuffer)
    this.elementOffset = StaticGeometry.addElements(indicesBuffer)

		gl.vertexAttribPointer(0, 3, gl.BYTE, false, 8, vertexOffet + 0)
    gl.enableVertexAttribArray(0)
    gl.vertexAttribPointer(1, 3, gl.BYTE, false, 8, vertexOffet + 3)
    gl.enableVertexAttribArray(1)
    gl.vertexAttribPointer(2, 2, gl.BYTE, false, 8, vertexOffet + 6)
    gl.enableVertexAttribArray(2)
  }

  static draw() {
		gl.bindVertexArray(this.vao)
		gl.drawElements(gl.TRIANGLES, 36, gl.UNSIGNED_BYTE, this.elementOffset)
  }

  static drawOutlines() {
    gl.bindVertexArray(this.vao)
    gl.drawElements(gl.LINES, 24, gl.UNSIGNED_BYTE, this.elementOffset + 36)
  }
}
*/