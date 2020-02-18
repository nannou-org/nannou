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
