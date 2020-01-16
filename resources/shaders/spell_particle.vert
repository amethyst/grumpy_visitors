#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
    uniform mat4 proj_view;
};

// Quad transform.
layout(location = 0) in vec2 pos;
layout(location = 1) in float seconds_since_spawn;

layout(location = 0) out VertexData {
    vec2 uv;
    float seconds_since_spawn;
} vertex;

const vec2 positions[4] = vec2[](
    vec2(0.5, -0.5), // Right bottom
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, 0.5), // Right top
    vec2(-0.5, 0.5) // Left top
);

const vec2 size = vec2(2.0);
const float z = 50.0;

void main() {
    float u = positions[gl_VertexIndex][0];
    float v = positions[gl_VertexIndex][1];

    vertex.uv = vec2(u, v) + vec2(0.5);
    vertex.seconds_since_spawn = seconds_since_spawn;
    vec2 final_pos = pos + vec2(u * size.x, v * size.y);
    vec4 vertex = vec4(final_pos, z, 1.0);
    gl_Position = proj_view * vertex;
}
