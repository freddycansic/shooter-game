#version 450

layout(location = 0) in vec3 position;
//layout(location = 1) in vec3 normal;
//layout(location = 2) in vec2 tex_coord;
//layout(location = 3) in vec3 camera_position;

layout(location = 0) out vec4 out_color;

struct Light {
    vec4 position;
    vec4 color;
    float intensity;
};

//layout(binding = 2) uniform LightsUniform {
//    Light lights[10];
//} lights_uniform;
//
//layout(binding = 3) uniform sampler2D texture_sampler;

void main() {
//    Light light = lights_uniform.lights[0];
//
//    vec3 light_color = light.color.rgb;
//
//    // ambient
//    float ambient_strength = 0.1;
//    vec3 ambient = ambient_strength * light_color;
//
//    vec4 color = texture(texture_sampler, tex_coord);
//
//    // diffuse
//    // how close is the angle of incidence to the normal?
//    vec3 light_direction = normalize(light.position.xyz - position);
//
//    float incidence_angle = max(dot(normal, light_direction), 0.0);
//    vec3 diffuse = incidence_angle * light_color;
//
//    // specular
//    // how close is the direction of reflected light to the direction from the fragment to the eye?
//    float specular_strength = 0.5;
//    vec3 view_direction = normalize(camera_position - position);
//    vec3 reflection_direction = reflect(-light_direction, normal);
//
//    float shininess = 64;
//    float specularity = pow(max(dot(view_direction, reflection_direction), 0.0), shininess);
//    vec3 specular = specular_strength * specularity * light_color;
//
//    out_color = vec4(ambient + diffuse + specular, 1.0) * color;

    out_color = vec4(position, 1.0);
}