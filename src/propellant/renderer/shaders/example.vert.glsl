#version 450

vec2 positions[3] = vec2[](
    vec2(-1.0, -1.0),   // Top left
    vec2(+3.0, -1.0),   // Top right
    vec2(-1.0, +3.0)   // Bottom left
);

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
