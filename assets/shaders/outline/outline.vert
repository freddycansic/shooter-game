#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

layout (location = 3) in mat4 transform;

uniform float outlining;
uniform mat4 vp;

void main() {
    gl_Position = vp * vec4(position + normal * outlining, 1.0f);
}
