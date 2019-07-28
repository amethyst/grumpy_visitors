#version 450

layout(std140, set = 0, binding = 0) uniform ViewArgs {
    uniform mat4 proj;
    uniform mat4 view;
} viewArgs;

layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 position;

layout(push_constant) uniform PushConstants {
    vec2 translation;
    float scale;
    float health;
} pushConstants;

layout(location = 0) out VertexData {
    vec2 uv;
} vertex;

void main()
{
    float camera_z = viewArgs.view[3].z;
    vec3 screen_pos = vec3(position.xy * pushConstants.scale + pushConstants.translation, camera_z + position.z);
    vec4 position = viewArgs.proj * vec4(screen_pos, 1.0);

    vertex.uv = uv;
    gl_Position = vec4(position);
}
