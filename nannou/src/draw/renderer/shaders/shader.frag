// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `frag.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o frag.spv shader.frag`

#version 450

layout(set = 1, binding = 0) uniform sampler text_sampler;
layout(set = 1, binding = 1) uniform texture2D text;
layout(set = 2, binding = 0) uniform sampler tex_sampler;
layout(set = 2, binding = 1) uniform texture2D tex;

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_tex_coords;
layout(location = 2) flat in uint v_mode;

layout(location = 0) out vec4 f_color;

void main() {
    vec4 f_color_texture = texture(sampler2D(tex, tex_sampler), v_tex_coords);
    float tex_a = texture(sampler2D(text, text_sampler), v_tex_coords).r;
    
    // Color
    if (v_mode == uint(0)) {
        f_color = v_color;
    // Texture
    } else if (v_mode == uint(1)) {
        f_color = f_color_texture;
    // Text
    } else if (v_mode == uint(2)) {
        f_color = vec4(v_color.rgb, v_color.a * tex_a);
    // Unhandled mode - Indicate error with red.
    } else {
        f_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
}
