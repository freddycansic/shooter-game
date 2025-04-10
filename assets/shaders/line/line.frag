#version 450

layout(location = 0) out vec4 out_color;

in VS_OUT {
    vec3 color;
} vs_in;

void main() {
    out_color = vec4(vs_in.color, 1.0);
}
