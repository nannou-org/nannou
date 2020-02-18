#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 tex_coords;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_tex_coords;

void main() {
    gl_Position = vec4(position, 1.0);
    v_color = color;
    v_tex_coords = tex_coords;
}
