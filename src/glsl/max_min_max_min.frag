#version 300 es
precision highp float;

layout(location = 0) out vec2 fragmentColor;

uniform sampler2D max_min_depth_texture;

// TODO: use texture instead of texelFetch, and get the bias calculated on the vertex shader, just because

void main() {
  ivec2 p = ivec2(gl_FragCoord.xy) * 2;

  vec2 a = texelFetch(max_min_depth_texture, p, 0).rg;
  vec2 b = texelFetchOffset(max_min_depth_texture, p, 0, ivec2(1, 0)).rg;
  vec2 c = texelFetchOffset(max_min_depth_texture, p, 0, ivec2(0, 1)).rg;
  vec2 d = texelFetchOffset(max_min_depth_texture, p, 0, ivec2(1, 1)).rg;

  fragmentColor = vec2(max(max(a.r, b.r), max(c.r, d.r)), min(min(a.g, b.g), min(c.g, d.g)));
}
