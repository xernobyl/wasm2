#version 300 es
precision highp float;

/*
TODO: calculate the correct shape using previous
and next points to fill the intersections
*/

layout(location = 0) in vec3 line_a_;
layout(location = 1) in vec3 line_a;
layout(location = 2) in vec3 line_b;
layout(location = 3) in vec3 line_b_;

void main() {
  /*
  const vec4 vertices = vec4(0.0, 1.0, -0.5, 0.5);
  vec2 vertex;

  switch (gl_VertexID) {
    case 0:
      vertex = vertices.rb;
      break;
    case 1:
      vertex = vertices.gb;
      break;
    case 2:
      vertex = vertices.ra;
      break;
    default:
      vertex = vertices.ga;
  };

  float line_width = line_a.z;

  vec2 x_basis = line_b.xy - line_a.xy;
  vec2 y_basis = line_width * normalize(vec2(-x_basis.y, x_basis.x));
  vertex = line_a.xy + vertex.x * x_basis + vertex.y * y_basis;
  gl_Position = vec4(vertex, 0.0, 1.0);
  */

  vec2 vertex;
  vec2 db = normalize(line_b.xy - line_a.xy);

  if (gl_VertexID == 0 || gl_VertexID == 2) {
    vec2 da = normalize(line_a.xy - line_a_.xy);
    vec2 perp = 0.5 * da + 0.5 * db;
    perp = vec2(-perp.y, perp.x) * (gl_VertexID == 0 ? -0.5 : 0.5) * line_a.z;

    vertex = line_a.xy + perp;
  }
  else {
    vec2 dc = normalize(line_b_.xy - line_b.xy);
    vec2 perp = 0.5 * db + 0.5 * dc;
    perp = vec2(-perp.y, perp.x) * (gl_VertexID == 1 ? -0.5 : 0.5) * line_b.z;

    vertex = line_b.xy + perp;
  }

  vertex.x *= 10.0 / 16.0;

  gl_Position = vec4(vertex, 0.0, 1.0);
}
