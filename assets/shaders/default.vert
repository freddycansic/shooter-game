#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;

layout(location = 0) out vec3 out_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec2 out_tex_coord;
layout(location = 3) out vec3 out_camera_position;

// TODO change these to use different sets, as they are updated at different frequencies
// per frame
layout(set = 0, binding = 0) uniform CameraUniform {
    mat4 view;
    mat4 projection;
    vec3 camera_position;
} camera_uniform;

// per mesh
layout(set = 0, binding = 1) uniform ModelUniform {
    mat4 model;
    mat4 normal;
} model_uniform;

void main() {
    out_position = position;
    // Fix non-uniform scalings
    out_normal = vec3(model_uniform.normal * vec4(normalize(normal), 1.0));
    out_tex_coord = tex_coord;
    out_camera_position = camera_uniform.camera_position;

    gl_Position = camera_uniform.projection * camera_uniform.view * model_uniform.model * vec4(position, 1.0);
}