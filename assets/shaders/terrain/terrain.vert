#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

uniform mat4 vp;

out VS_OUT {
    vec3 position;
    vec3 normal;
} vs_out;

void main() {
    vs_out.position = position;
    vs_out.normal = normalize(normal);

    gl_Position = vp * vec4(position, 1.0);
}