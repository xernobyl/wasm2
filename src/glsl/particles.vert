#version 300 es
precision highp float;

/*
Draw an hexagon:
   2---4
  /|\  |\
0  | \ | 5
  \|  \|/
   1---3

Hexagon contains circle of radius = 0.5.
*/

layout(location = 0) in vec3 position;

const float N = 0.5 * tan(3.1415926535897932384626433832795 / 6.0);
const float H = 0.5 / cos(3.1415926535897932384626433832795 / 6.0);

void main() {
  const vec4 vertices0 = vec4(0.0, 0.5, N, H);
  const vec2 vertices1 = vec2(N, -0.5);

  vec2 vertex;

  switch (gl_VertexID) {
    case 0:
      vertex = -vertices0.ar;  // vertex = vec2(-H, 0.0);
      break;
    case 1:
      vertex = -vertices0.bg;  // vertex = vec2(-N, -0.5);
      break;
    case 2:
      vertex = -vertices1.rg;  // vec2(-N, 0.5);
      break;
    case 3:
      vertex = vertices1.rg;  // vec2(N, -0.5);
      break;

    case 4:
      vertex = vertices0.bg; // vertex = vec2(N, 0.5);
      break;

    default:
      vertex = vertices0.ar; // vertex = vec2(H, 0.0);
  };

  vec2 uv = vertex + 0.5;
  float r = clamp(length(vertex), 0.0, 0.5);

  vec3 position = vec3(vertex, 0.0);

  gl_Position = vec4(position, 1.0);
}
