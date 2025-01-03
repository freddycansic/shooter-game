#version 450

// Model
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

// Instance
layout (location = 3) in mat4 transform;

out VS_OUT {
    vec3 position;
    vec2 tex_coord;
    vec3 normal;
} vs_out;

// TODO if anything bad happens listen to this guy https://stackoverflow.com/questions/38172696/should-i-ever-use-a-vec3-inside-of-a-uniform-buffer-or-shader-storage-buffer-o

// per frame
uniform mat4 vp;

void main() {
    vs_out.position = position;
    vs_out.tex_coord = tex_coord;

    // TODO move calculation to uniform
    vs_out.normal = normalize(transpose(inverse(mat3(transform))) * normal);

    gl_Position = vp * transform * vec4(position, 1.0);
}