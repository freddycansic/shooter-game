#version 450

out VS_OUT {
    vec2 tex_coord;
} vs_out;

void main() {
    const vec2 corners[4] = vec2[](
            vec2(-1.0, -1.0),
            vec2(+1.0, -1.0),
            vec2(-1.0, +1.0),
            vec2(+1.0, +1.0)
        );

    const vec2 tex_coords[4] = vec2[](
            vec2(0.0, 0.0),
            vec2(1.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0)
        );

    vs_out.tex_coord = tex_coords[gl_VertexID];
    gl_Position = vec4(corners[gl_VertexID], 0.0, 1.0);
}
