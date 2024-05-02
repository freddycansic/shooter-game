#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;
layout(location = 3) in mat4 transform;
layout(location = 4) in mat4 transform_normal;

layout(location = 0) out vec3 out_position;
//layout(location = 1) out vec3 out_normal;
//layout(location = 2) out vec2 out_tex_coord;
//layout(location = 3) out vec3 out_camera_position;

// TODO if anything bad happens listen to this guy https://stackoverflow.com/questions/38172696/should-i-ever-use-a-vec3-inside-of-a-uniform-buffer-or-shader-storage-buffer-o

// per frame
uniform mat4 view;
uniform mat4 projection;
uniform vec3 camera_position;

void main() {
    out_position = position;
//    // Fix non-uniform scalings
//    out_normal = vec3(model_uniform.normal * vec4(normalize(normal), 1.0));
//    out_tex_coord = tex_coord;
//    out_camera_position = camera_uniform.camera_position;
//
    gl_Position = projection * view * transform * vec4(position, 1.0);
}