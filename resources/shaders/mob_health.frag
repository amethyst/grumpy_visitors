#version 450

layout(location = 0) in VertexData {
    vec2 uv;
    float health;
    float bar_ratio;
} vertex;
layout(location = 0) out vec4 out_color;

const vec3 healthy_color = vec3(0.0, 1.0, 0.0);
const vec3 unhealthy_color = vec3(1.0, 0.0, 0.0);

void main() {
    vec2 uv = vertex.uv;

    float border_height = 0.25; // relative to height
    float border_width = border_height / vertex.bar_ratio;
    float is_border = max(
        1.0 - step(border_width, mod(uv.x, 1.0 - border_width)),
        1.0 - step(border_height, mod(uv.y, 1.0 - border_height))
    );

    vec4 border_color = vec4(0.0, 0.0, 0.0, 1.0);
    border_color = vec4((1.0 - uv.x) * unhealthy_color * 0.1 + uv.x * healthy_color * 0.05, 1.0);
    vec4 bar_color = vec4(
        (1.0 - vertex.health) * unhealthy_color + vertex.health * healthy_color,
        1.0 - step(vertex.health, uv.x)
    );

    out_color = is_border * border_color + (1.0 - is_border) * bar_color;
}
