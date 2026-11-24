#version 450

layout(location = 0) out vec2 texture_uv;

layout(push_constant) uniform WallpaperUv {
    vec4 uv_rect;
} wallpaper_uv;

const vec2 POSITIONS[3] = vec2[](
    vec2(-1.0, -1.0),
    vec2( 3.0, -1.0),
    vec2(-1.0,  3.0)
);

void main() {
    vec2 position = POSITIONS[gl_VertexIndex];
    vec2 unit_uv = position * 0.5 + 0.5;
    gl_Position = vec4(position, 0.0, 1.0);
    texture_uv = mix(
        wallpaper_uv.uv_rect.xy,
        wallpaper_uv.uv_rect.zw,
        unit_uv
    );
}
