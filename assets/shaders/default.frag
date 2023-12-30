#version 450

layout(location = 0) in vec3 normal;
layout(location = 1) in vec3 position;
layout(location = 2) in vec3 camera_position;

layout(location = 0) out vec4 out_color;

struct Light {
    vec3 position;
    vec3 color;
    float intensity;
};

layout(set = 0, binding = 2) uniform LightsUniform {
    Light lights[10];
} lights_uniform;

void main() {
    Light light = lights_uniform.lights[0];

    // ambient
    float ambient_strength = 0.1;
    vec3 ambient = ambient_strength * light.color;

    // TODO
    vec3 color = vec3(1.0, 0.0, 0.0);

    // diffuse
    // how close is the angle of incidence to the normal?
    vec3 light_direction = normalize(light.position - position);

    float incidence_angle = max(dot(normal, light_direction), 0.0);
    vec3 diffuse = incidence_angle * light.color;

    // specular
    // how close is the direction of reflected light to the direction from the fragment to the eye?
    float specular_strength = 0.5;
    vec3 view_direction = normalize(camera_position - position);
    vec3 reflection_direction = reflect(-light_direction, normal);

    float shininess = 64;
    float specularity = pow(max(dot(view_direction, reflection_direction), 0.0), shininess);
    vec3 specular = specular_strength * specularity * light.color;

    vec3 result = (ambient + diffuse + specular) * color;

    out_color = vec4(result, 1.0);
}