#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 tex_uv;
    vec4 color;
} vertex;
layout(location = 0) out vec4 out_color;

void main() {
    vec3 brightest_white = vec3(0.76);
    vec3 darkest_white = vec3(0.57);
    vec3 white_diff = brightest_white - darkest_white;

    vec3 base_color = vec3(0.0, 0.49, 0.26);

    vec4 color = texture(albedo, vertex.tex_uv) * vertex.color;
    if (color.a == 0.0) {
        discard;
    }
    if (abs(color.r - color.g) < 0.02 && abs(color.g - color.b) < 0.02) {
        out_color = vec4(base_color - (brightest_white - color.rgb), color.a);
    } else {
        out_color = color;
    }
}
