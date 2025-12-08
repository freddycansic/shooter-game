#version 450

// Model
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;

// Instance
layout(location = 3) in mat4 transform;

uniform mat4 vp;

void main() {
    gl_Position = vp * transform * vec4(position, 1.0);
}
