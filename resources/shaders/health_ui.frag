#version 450

layout(push_constant) uniform FragmentPushConstants {
    layout(offset = 12)
    float health;
} pushConstants;

layout(location = 0) in VertexData {
    vec2 uv;
} vertex;

layout(location = 0) out vec4 outColor;

const vec3 border_color = vec3(0.001);
const vec3 blood_color = vec3(0.8, 0.025, 0.027);

void main()
{
    vec2 d = vertex.uv - vec2(0.5);
    float r = dot(d, d) * 4.0;
    float a = 1.0 - smoothstep(0.98, 1.0, r);

    vec3 mix_c = mix(border_color, blood_color, 1.0 - smoothstep(0.0, 0.1, vertex.uv.y - pushConstants.health));
    vec4 remained_blood_color = vec4(mix_c, 1.0 - smoothstep(0.0, 0.1, vertex.uv.y - pushConstants.health));

    vec2 shadow_d = vertex.uv - vec2(0.65, 0.60);
    float shadow_r = dot(shadow_d, shadow_d) * 4.0;
    vec4 shadowed_color = mix(remained_blood_color, vec4(border_color, a), smoothstep(0.1, 1.5, shadow_r - 0.05));

    vec4 color = mix(shadowed_color, vec4(border_color, a), smoothstep(0.925, 1.0, r));
    outColor = vec4(color);
}
