#version 300 es
precision highp float;

in vec2 uv;
layout(location = 0) out vec3 fragmentColor;

uniform sampler2D color_texture;

vec3 linear_to_srgb(vec3 linear_rgb) {
  vec3 a = 12.92 * linear_rgb;
  vec3 b = 1.055 * pow(linear_rgb, vec3(1.0 / 2.4)) - 0.055;
  vec3 c = step(vec3(0.0031308), linear_rgb);
  return mix(a, b, c);
}

void main() {
  /*vec2 p = uv - 0.5;
  float r = length(p);
  float a = atan(p.y, p.x);
  r = r * r * 3.0;
  p = r * vec2(cos(a) * 0.5, sin(a) * 0.5);

  vec3 color = texture(color_texture, p + 0.5).rgb;
  fragmentColor = linear_to_srgb(color);*/

  fragmentColor = linear_to_srgb(texture(color_texture, uv).rgb);
}
