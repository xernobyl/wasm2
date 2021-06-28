#version 300 es
precision highp float;

layout(location = 0) out vec3 fragmentColor;

in vec3 pos;

vec3 box_gradient(vec3 p) {
  vec3 d = abs(p) - vec3(1.0);
  vec3 s = vec3(p.x < 0.0 ? -1.0 : 1.0,
                p.y < 0.0 ? -1.0 : 1.0,
                p.z < 0.0 ? -1.0 : 1.0);
  float g = max(d.x, max(d.y, d.z));
  return s * (g > 0.0 ? normalize(max(d, 0.0)) :
                        step(d.yzx, d.xyz) * step(d.zxy, d.xyz));
}

void main() {
  fragmentColor = pos * 0.5 + 0.5;
  // vec3 norm = box_gradient(pos * 2.0);
  // fragmentColor = norm * 0.5 + 0.5;
}
