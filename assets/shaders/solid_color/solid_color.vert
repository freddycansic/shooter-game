#version 450

// Model
layout(location = 0) in vec3 position;

// Instance
layout(location = 1) in vec4 transform_x;
layout(location = 2) in vec4 transform_y;
layout(location = 3) in vec4 transform_z;
layout(location = 4) in vec4 transform_w;
layout(location = 5) in vec3 color;

uniform mat4 vp;

out VS_OUT {
    vec3 color;
} vs_out;

void main() {
    vs_out.color = color;

    mat4 transform = mat4(transform_x, transform_y, transform_z, transform_w);

    gl_Position = vp * transform * vec4(position, 1.0);
}
