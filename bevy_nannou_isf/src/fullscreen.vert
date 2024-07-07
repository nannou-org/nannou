#version 450

layout(location = 0) out vec2 isf_FragNormCoord;

void main() {
    uint vertex_index = gl_VertexIndex;

    // Calculate UV coordinates
    isf_FragNormCoord = vec2(float(vertex_index >> 1), float(vertex_index & 1)) * 2.0;

    // Calculate clip position
    vec2 clip_position = isf_FragNormCoord * vec2(2.0, -2.0) + vec2(-1.0, 1.0);
    gl_Position = vec4(clip_position, 0.0, 1.0);
}