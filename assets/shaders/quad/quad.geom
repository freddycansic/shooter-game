#version 450

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

in vec2 vs_out_size[];
out vec2 gs_out_tex_coord;

void main() {
    // Bottom right
    gl_Position = vec4(gl_in[0].gl_Position.x + vs_out_size[0].x, gl_in[0].gl_Position.y, 0.0, 1.0);
    gs_out_tex_coord = vec2(1.0, 0.0);
    EmitVertex();

    // Bottom left
    gl_Position = gl_in[0].gl_Position;
    gs_out_tex_coord = vec2(0.0, 0.0);
    EmitVertex();

    // Top right
    gl_Position = vec4(gl_in[0].gl_Position.x + vs_out_size[0].x, gl_in[0].gl_Position.y + vs_out_size[0].y, 0.0, 1.0);
    gs_out_tex_coord = vec2(1.0, 1.0);
    EmitVertex();

    // Top left
    gl_Position = vec4(gl_in[0].gl_Position.x, gl_in[0].gl_Position.y + vs_out_size[0].y, 0.0, 1.0);
    gs_out_tex_coord = vec2(0.0, 1.0);
    EmitVertex();

    EndPrimitive();
}