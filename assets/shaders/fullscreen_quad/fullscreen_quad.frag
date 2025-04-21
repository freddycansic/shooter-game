#version 450

layout(location = 0) out vec4 out_color;

in VS_OUT {
    vec2 tex_coord;
} vs_in;

uniform sampler2D fullscreen_quad_texture;

void main() {
    out_color = texture(fullscreen_quad_texture, vs_in.tex_coord);
}
