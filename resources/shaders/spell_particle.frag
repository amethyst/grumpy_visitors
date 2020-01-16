#version 450

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 uv;
    float seconds_since_spawn;
} vertex;
layout(location = 0) out vec4 out_color;

const float partcile_ttl = 0.25;
const vec3 particle_color = vec3(1.0, 0.0, 0.0);

void main() {
    out_color = vec4(particle_color, max(1.0 - vertex.seconds_since_spawn / partcile_ttl, 0.0));
}
