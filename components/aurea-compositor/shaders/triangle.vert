#version 450

layout(location = 0) out vec3 vertex_color;

const vec2 POSITIONS[3] = vec2[](
    vec2( 0.0, -0.65),
    vec2( 0.65, 0.55),
    vec2(-0.65, 0.55)
);

const vec3 COLORS[3] = vec3[](
    vec3(0.95, 0.25, 0.20),
    vec3(0.20, 0.80, 0.35),
    vec3(0.20, 0.45, 0.95)
);

void main() {
    gl_Position = vec4(POSITIONS[gl_VertexIndex], 0.0, 1.0);
    vertex_color = COLORS[gl_VertexIndex];
}
