#version 450

layout(set = 0, binding = 0) uniform sampler2D cursor_texture;
layout(location = 0) in vec2 texture_uv;
layout(location = 0) out vec4 fragment_color;

layout(push_constant) uniform CursorTrailOpacity {
    layout(offset = 4) float opacity;
} trail;

void main() {
    vec4 cursor = texture(cursor_texture, texture_uv);
    fragment_color = vec4(cursor.rgb * trail.opacity, cursor.a * trail.opacity);
}
