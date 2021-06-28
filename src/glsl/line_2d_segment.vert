#version 300 es
precision highp float;

layout(location = 0) in vec3 line_a;
layout(location = 1) in vec3 line_b;

void main() {
  vec2 vertex;
  vec2 x_basis = line_b.xy - line_a.xy;
  vec2 y_basis = normalize(vec2(-x_basis.y, x_basis.x));

  if (gl_VertexID == 0 || gl_VertexID == 2) {
    vertex = line_a.xy + y_basis * (gl_VertexID == 0 ? -0.5 : 0.5) * line_a.z;
  }
  else {
    vertex = line_b.xy + y_basis * (gl_VertexID == 1 ? -0.5 : 0.5) * line_b.z;
  }

  gl_Position = vec4(vertex, 0.0, 1.0);
}
