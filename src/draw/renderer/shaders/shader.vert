// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `vert.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o vert.spv shader.vert`

#version 450

layout(set = 0, binding = 0) uniform Data {
    vec3 window_to_shader;
} uniforms;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 tex_coords;
layout(location = 3) in uint mode;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_tex_coords;
layout(location = 2) flat out uint v_mode;

void main() {
    gl_Position = vec4(position * uniforms.window_to_shader, 1.0);
    v_color = color;
    v_tex_coords = tex_coords;
    v_mode = mode;
}
