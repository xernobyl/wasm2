#version 300 es
precision highp float;

const vec2 vertices[3] = vec2[3](
  vec2(-1.0, -1.0),
  vec2(3.0, -1.0),
  vec2(-1.0, 3.0)
);

out vec2 uv;

void main() {
  gl_Position = vec4(vertices[gl_VertexID], 0.0, 1.0);
  uv = gl_Position.xy * 0.5 + 0.5;
}
