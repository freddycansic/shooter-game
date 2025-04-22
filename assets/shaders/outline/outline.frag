#version 450

layout(location = 0) out vec4 out_color;

in VS_OUT {
    vec2 tex_coord;
} vs_in;

uniform sampler2D mask_texture;
uniform vec3 outline_color;
uniform int outline_radius;

void main() {
    float current = texture(mask_texture, vs_in.tex_coord).r;
    ivec2 texture_dimensions = textureSize(mask_texture, 0);
    vec2 pixel_size = 1.0 / vec2(texture_dimensions);
    bool dilate = false;

    for (int x = -outline_radius; x <= outline_radius; x++) {
        for (int y = -outline_radius; y <= outline_radius; y++) {
            vec2 neighbour_coordinates = vs_in.tex_coord + vec2(x, y) * pixel_size;

            // If one of the neighbours has colour, then dilate
            if (texture(mask_texture, neighbour_coordinates).r > 0.0) {
                dilate = true;
                break;
            }
        }

        if (dilate) {
            break;
        }
    }

    if (dilate && current <= 0.0) {
        out_color = vec4(outline_color, 1.0);
    } else {
        out_color = vec4(0.0);
    }
}
