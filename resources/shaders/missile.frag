#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 uv;
    float seconds_since_spawn;
} vertex;
layout(location = 0) out vec4 out_color;

const float PI = 3.14159265358979323846;

const vec3 missile_light_color = vec3(1.0, 0.0, 0.0);
const vec3 missile_ray_color = vec3(0.7, 0.7, 0.7);

// y = (1.0 - x) * 2.7 / 40.0; x ∈ [0.0; 1.0 / 2.7]
const float x_constraint = 1.0 / 2.7;
const float y_constraint = 1.0 / 40.0;

// Global speed multiplier.
const float gs = 0.4;

vec4 ray_color(vec2 uv, float size_multiplier) {
    float y = (1.0 - abs(uv.x) / (x_constraint * size_multiplier)) * y_constraint * size_multiplier;
    float x_alpha = smoothstep(0.0, x_constraint * size_multiplier, abs(uv.x));
    float y_alpha = smoothstep(0.0, y, abs(uv.y));
    float max_a = max(x_alpha, y_alpha);

    float mix_x = 1.0 - abs(uv.x) / x_constraint;
    float mix_y = 1.0 - abs(uv.y) / y;
    return vec4(mix(missile_light_color, missile_ray_color, min(mix_x, mix_y) - 0.6), 1.0 - max_a);
}

vec2 rotate(float angle, vec2 uv) {
    return vec2(uv.x * cos(angle) - uv.y * sin(angle), uv.y * cos(angle) + uv.x * sin(angle));
}

void main() {
    vec2 d = vertex.uv - vec2(0.5);
    float r = dot(d, d) * 4.0;
    float a = 0.1 * (1.0 - smoothstep(0.1, 1.0, r));

    float s = vertex.seconds_since_spawn;
    vec2 uv = vertex.uv - vec2(0.5);
    vec4 ray_1 = ray_color(rotate(PI * mod(s * gs * 1.5, 1.0) + PI * 0.0, uv), 1.0);
    vec4 ray_2 = ray_color(rotate(PI * mod(s * gs * 1.5, 1.0) + PI * 0.16, uv), 0.7);
    vec4 ray_3 = ray_color(rotate(PI * mod(s * gs * 1.8, 1.0) + PI * 0.33, uv), 0.8);
    vec4 ray_4 = ray_color(rotate(PI * mod(s * gs * 1.9, 1.0) + PI * 0.5, uv), 1.0);
    vec4 ray_5 = ray_color(rotate(PI * mod(s * gs * 1.5, 1.0) + PI * 0.66, uv), 0.7);
    vec4 ray_6 = ray_color(rotate(PI * mod(s * gs * 1.8, 1.0) + PI * 0.83, uv), 0.65);
    vec4 ray_7 = ray_color(rotate(PI * mod(s * gs * 1.6, 1.0) + PI * 0.2, uv), 0.5);
    vec4 ray_8 = ray_color(rotate(PI * mod(s * gs * 1.8, 1.0) + PI * 0.7, uv), 0.5);

    vec4 rays_combined = vec4(
        max(max(max(max(max(max(max(
                ray_1,
                ray_2),
                ray_3),
                ray_4),
                ray_5),
                ray_6),
                ray_7),
                ray_8)
    );
    out_color = mix(rays_combined, vec4(missile_light_color, a), 1.0 - rays_combined.a);
}
