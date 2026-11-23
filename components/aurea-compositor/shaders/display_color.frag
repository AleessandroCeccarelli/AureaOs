#version 450

layout(set = 0, binding = 0) uniform sampler2D composed_scene;
layout(push_constant) uniform DisplayColorParameters {
    float warmth;
} parameters;

layout(location = 0) in vec2 texture_uv;
layout(location = 0) out vec4 fragment_color;

void main() {
    vec4 source = texture(composed_scene, texture_uv);
    float strength = clamp(parameters.warmth, 0.0, 1.0);
    vec3 warm = vec3(
        min(source.r * 1.035 + 0.018, 1.0),
        source.g * 0.955 + 0.008,
        source.b * 0.78
    );
    fragment_color = vec4(mix(source.rgb, warm, strength), 1.0);
}
