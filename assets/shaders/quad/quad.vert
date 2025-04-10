#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 size;
layout(location = 2) in int layer;

out vec2 vs_out_size;

void main() {
    // Add 50 to the layer so that 0 starts at -50
    // Then layer 1 would equate to -49
    // and layer -1 would equate to -51
    gl_Position = vec4(position, float(-(layer + 50)), 1.0);
    vs_out_size = size;
}
