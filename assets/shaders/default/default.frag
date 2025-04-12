#version 450

in VS_OUT {
    vec3 position;
    vec2 tex_coord;
    vec3 normal;
} vs_in;

layout(location = 0) out vec4 out_color;

uniform sampler2D diffuse_texture;
uniform sampler2D specular_texture;
uniform vec3 light_color;
uniform vec3 light_position;
uniform vec3 camera_position;

void main() {
    // Ambient
    float ambient_strength = 0.3;
    vec3 ambient = ambient_strength * light_color;

    // Diffuse
    vec3 light_direction = normalize(light_position - vs_in.position);

    float diffuse_strength = max(dot(vs_in.normal, light_direction), 0.0);
    vec3 diffuse = diffuse_strength * light_color;

    // Specular
    vec3 view_direction = normalize(camera_position - vs_in.position);
    vec3 reflect_direction = reflect(-light_direction, vs_in.normal);

    vec4 specular_color = texture(specular_texture, vs_in.tex_coord);
    int shininess = 32;
    float specular_factor = pow(max(dot(view_direction, reflect_direction), 0.0), shininess);
    vec3 specular = specular_color.xyz * specular_factor * light_color;

    vec4 diffuse_color = texture(diffuse_texture, vs_in.tex_coord);

    out_color = diffuse_color * vec4((ambient + diffuse + specular), 1.0);
    // out_color = vec4(1.0, 1.0, 1.0, 1.0);
}
