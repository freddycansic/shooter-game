#version 450

// Model
layout(location = 0) in vec3 position;

// Instance
layout(location = 1) in mat4 transform;
layout(location = 5) in vec3 color;

uniform mat4 vp;

out VS_OUT {
    vec3 color;
} vs_out;

void main() {
    vs_out.color = color;

    gl_Position = vp * transform * vec4(position, 1.0);
}
