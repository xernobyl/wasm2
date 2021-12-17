#version 300 es
precision highp float;

layout(location = 0) out vec3 fragmentColor;

in float width;

void main() {
  fragmentColor = vec3(abs(fract(width * 10.0)));
}
