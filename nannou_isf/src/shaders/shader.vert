// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `vert.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o vert.spv shader.vert`

#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 isf_FragNormCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    isf_FragNormCoord = vec2(position.x * 0.5 + 0.5, 1.0 - (position.y * 0.5 + 0.5));
}
