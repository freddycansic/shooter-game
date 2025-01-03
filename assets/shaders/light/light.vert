#version 450

// Model
layout (location = 0) in vec3 position;

// Instance
layout (location = 1) in vec3 light_translation;
layout (location = 2) in vec3 light_color;

out VS_OUT {
    vec3 color;
} vs_out;

uniform mat4 vp;

void main() {
    vs_out.color = light_color;

    gl_Position = vp * vec4(position * 0.2 + light_translation, 1.0);
}