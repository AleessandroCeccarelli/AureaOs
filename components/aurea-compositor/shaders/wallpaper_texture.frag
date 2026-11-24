#version 450

layout(set = 0, binding = 0) uniform sampler2D wallpaper_texture;
layout(location = 0) in vec2 texture_uv;
layout(location = 0) out vec4 fragment_color;

void main() {
    fragment_color = texture(wallpaper_texture, texture_uv);
}
