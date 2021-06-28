#version 300 es
precision highp float;

layout(location = 0) out vec2 fragmentColor;

uniform sampler2D depth_texture;

void main() {
  ivec2 p = ivec2(gl_FragCoord.xy) * 2;

  float a = texelFetch(depth_texture, p, 0).r;
  float b = texelFetchOffset(depth_texture, p, 0, ivec2(1, 0)).r;
  float c = texelFetchOffset(depth_texture, p, 0, ivec2(0, 1)).r;
  float d = texelFetchOffset(depth_texture, p, 0, ivec2(1, 1)).r;

  fragmentColor = vec2(max(max(a, b), max(c, d)), min(min(a, b), min(c, d)));
}
