#version 300 es
precision highp float;

layout(location = 0) in vec3 vertex;
layout(location = 1) in mat4 transform;
layout(location = 5) in vec3 position;

uniform mat4 camera;
uniform vec3 camera_position;

out vec3 pos;

void main() {
  pos = vertex;
  gl_Position = transform * vec4(vertex, 1.0);


  /*vec3 t = position - camera_position;

  pos = vec3(t.x < 0.0 ? vertex.x : -vertex.x,  // 1
             t.y < 0.0 ? -vertex.y : vertex.y,  // 1
             t.z < 0.0 ? -vertex.z : vertex.z);

  pos = vertex;

  gl_Position = camera * vec4(pos, 1.0);*/
}
