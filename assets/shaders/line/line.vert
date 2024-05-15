#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 color;

out VS_OUT {
    vec3 color;
} vs_out;

uniform mat4 vp;

void main() {
    vs_out.color = color;

    gl_Position = vp * vec4(position, 1.0);
}