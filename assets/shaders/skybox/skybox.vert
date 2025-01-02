#version 450

layout (location = 0) in vec3 position;

out VS_OUT {
    vec3 tex_coord;
} vs_out;

uniform mat4 vp;

void main()
{
    vs_out.tex_coord = position;
    gl_Position = vp * vec4(position, 1.0);
}