#version 450

layout (location = 0) out vec4 out_color;

uniform vec3 camera_position;

in VS_OUT {
    vec3 position;
    vec3 normal;
} vs_in;

void main() {
    vec3 light_color = vec3(1.0, 1.0, 1.0);
    vec3 light_position = camera_position;
    vec4 diffuse_color = vec4(1.0, 1.0, 1.0, 1.0);

    // Ambient
    float ambient_strength = 0.3;
    vec3 ambient = ambient_strength * light_color;

    // Diffuse
    vec3 light_direction = normalize(light_position - vs_in.position);

    float diffuse_strength = max(dot(vs_in.normal, light_direction), 0.0);
    vec3 diffuse = diffuse_strength * light_color;

    // Combine
    out_color = diffuse_color * vec4((ambient + diffuse), 1.0);
}