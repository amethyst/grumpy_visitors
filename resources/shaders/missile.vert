#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 proj_view;
};

// Quad transform.
layout(location = 0) in vec2 pos;
layout(location = 1) in float seconds_since_spawn;
layout(location = 2) in float opacity;
layout(location = 3) in float ttl;

layout(location = 0) out VertexData {
    vec2 uv;
    float opacity;
    float seconds_since_spawn;
} vertex;

const vec2 positions[4] = vec2[](
    vec2(0.5, -0.5), // Right bottom
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, 0.5), // Right top
    vec2(-0.5, 0.5) // Left top
);

const vec2 base_size = vec2(70.0);
const float max_modifier = 2.0;
const float max_size_at = 0.4;
const float z = 50.0;

void main() {
    float u = positions[gl_VertexIndex][0];
    float v = positions[gl_VertexIndex][1];

    vec2 size;
    float death_time = 1.0 - ttl;
    if (death_time < max_size_at) {
        float growing_time = max_size_at;
        size = base_size * (1.0 + (max_modifier - 1.0) * (death_time / growing_time));
    } else {
        float fading_time = 1.0 - max_size_at;
        size = base_size * max_modifier * (1.0 - (death_time - max_size_at) / fading_time);
    }

    vertex.uv = vec2(u, v) + vec2(0.5);
    vertex.opacity = opacity;
    vertex.seconds_since_spawn = seconds_since_spawn;
    vec2 final_pos = pos + vec2(u * size.x, v * size.y);
    vec4 vertex = vec4(final_pos, z, 1.0);
    gl_Position = proj_view * vertex;
}
