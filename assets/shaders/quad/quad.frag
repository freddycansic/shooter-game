#version 450

layout (location = 0) out vec4 out_color;

in vec2 gs_out_tex_coord;

uniform sampler2D diffuse_texture;

void main() {
    out_color = texture(diffuse_texture, gs_out_tex_coord);
}