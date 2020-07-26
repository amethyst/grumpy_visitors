#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
} viewArgs;

layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 position;
layout(location = 2) in vec2 translation;
layout(location = 3) in float scale;
layout(location = 4) in float health;

layout(location = 0) out VertexData {
    vec2 uv;
    float health;
} vertex;

void main()
{
    float camera_z = viewArgs.view[3].z;
    vec3 screen_pos = vec3(position.xy * scale + translation, camera_z + position.z);
    vec4 position = viewArgs.proj * vec4(screen_pos, 1.0);

    vertex.uv = uv;
    vertex.health = health;

    gl_Position = vec4(position.xy, 0.5, 1.0);
}
