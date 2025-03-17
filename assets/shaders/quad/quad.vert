#version 450

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 size;

out vec2 vs_out_size;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vs_out_size = size;
}