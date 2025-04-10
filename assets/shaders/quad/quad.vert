#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 size;
layout(location = 2) in int layer;

out vec2 vs_out_size;

void main() {
    float factor = 0.001;
    gl_Position = vec4(position, float(-layer) * factor, 1.0);
    vs_out_size = size;
}
