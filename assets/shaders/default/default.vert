#version 450

// Model
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

// Instance
layout (location = 3) in mat4 transform;
layout (location = 4) in mat4 transform_normal;

out VS_OUT {
    vec3 position;
    vec2 tex_coord;
    //vec3 out_camera_position
} vs_out;

// TODO if anything bad happens listen to this guy https://stackoverflow.com/questions/38172696/should-i-ever-use-a-vec3-inside-of-a-uniform-buffer-or-shader-storage-buffer-o

// per frame
uniform mat4 vp;
uniform vec3 camera_position;

void main() {
    vs_out.position = position;
    vs_out.tex_coord = tex_coord;

    //    // Fix non-uniform scalings
    //    out_normal = vec3(model_uniform.normal * vec4(normalize(normal), 1.0));
    //    out_camera_position = camera_uniform.camera_position;

    gl_Position = vp * transform * vec4(position, 1.0);
}