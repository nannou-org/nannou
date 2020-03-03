// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `frag.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o frag.spv shader.frag`

#version 450

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_tex_coords;

layout(location = 0) out vec4 f_color;

void main() {
    // // Text
    // if (v_mode == uint(0)) {
    //     f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
    // // Image
    // } else if (v_mode == uint(1)) {
    //     f_color = texture(tex, v_tex_coords);
    // // 2D Geometry
    // } else if (v_mode == uint(2)) {
    //     f_color = v_color;
    // }
    f_color = v_color;
}
