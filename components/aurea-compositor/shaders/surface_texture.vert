#version 450

layout(location = 0) out vec2 texture_uv;

layout(push_constant) uniform SurfaceTransform {
    uint transform;
} surface_transform;

const vec2 POSITIONS[3] = vec2[](
    vec2(-1.0, -1.0),
    vec2( 3.0, -1.0),
    vec2(-1.0,  3.0)
);

vec2 transformed_uv(vec2 uv) {
    switch (surface_transform.transform) {
        case 1u: return vec2(uv.y, 1.0 - uv.x);
        case 2u: return vec2(1.0 - uv.x, 1.0 - uv.y);
        case 3u: return vec2(1.0 - uv.y, uv.x);
        case 4u: return vec2(1.0 - uv.x, uv.y);
        case 5u: return vec2(uv.y, uv.x);
        case 6u: return vec2(uv.x, 1.0 - uv.y);
        case 7u: return vec2(1.0 - uv.y, 1.0 - uv.x);
        default: return uv;
    }
}

void main() {
    vec2 position = POSITIONS[gl_VertexIndex];
    gl_Position = vec4(position, 0.0, 1.0);
    texture_uv = transformed_uv(position * 0.5 + 0.5);
}
