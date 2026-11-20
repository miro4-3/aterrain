#version 330 core

in uint a_PackedVertex;
uniform mat4 u_MVP;
uniform vec4 u_ChunkPos;

out vec3 v_color;

void main() {
  uint x = a_PackedVertex & 31u;
  uint y = (a_PackedVertex >> 5u) & 31u;
  uint z = (a_PackedVertex >> 10u) & 31u;
  uint face = (a_PackedVertex >> 15u) & 7u;

  vec3 position = vec3(float(x), float(y), float(z)) + u_ChunkPos.xyz * 16.0;
  gl_Position = u_MVP * vec4(position, 1.0);
  gl_Position.z = -gl_Position.z;
  uint axis = face / 2u;
  if (axis == 0u) {
    v_color = vec3(0.0, 1.0, 0.0);
  } else if (axis == 1u) {
    v_color = vec3(1.0, 0.1, 0.1);
  } else {
    v_color = vec3(0.1, 0.1, 1.0);
  }
}
