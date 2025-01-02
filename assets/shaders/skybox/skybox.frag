#version 450

layout (location = 0) out vec4 out_color;

in VS_OUT {
    vec3 tex_coord;
} vs_in;

uniform samplerCube skybox;

void main() {
    out_color = texture(skybox, vs_in.tex_coord);
}