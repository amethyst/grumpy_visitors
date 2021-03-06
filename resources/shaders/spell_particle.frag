#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 uv;
    float opacity;
    float seconds_since_spawn;
} vertex;
layout(location = 0) out vec4 out_color;

const float partcile_ttl = 0.25;
const vec3 particle_color = vec3(0.95, 0.3, 0.3);

void main() {
    out_color = vec4(particle_color, max(1.0 - vertex.seconds_since_spawn / partcile_ttl, 0.0) * vertex.opacity);
}
