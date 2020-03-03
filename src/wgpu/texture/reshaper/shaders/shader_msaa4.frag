// NOTE: This shader requires being manually compiled to SPIR-V in order to
// avoid having downstream users require building shaderc and compiling the
// shader themselves. If you update this shader, be sure to also re-compile it
// and update `frag_msaa4.spv`. You can do so using `glslangValidator` with the
// following command: `glslangValidator -V -o frag_msaa4.spv shader_msaa4.frag`

#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2DMS tex;
layout(set = 0, binding = 1) uniform sampler tex_sampler;

void main() {
    // Get the integer tex coordinates.
    ivec2 tex_size = textureSize(sampler2DMS(tex, tex_sampler));
    int tex_x = int(tex_size.x * tex_coords.x);
    int tex_y = int(tex_size.y * tex_coords.y);
    ivec2 itex_coords = ivec2(tex_x, tex_y);

    // Manually unroll the resolve. The less conditions the better!
    vec4 color = vec4(0);
    color += texelFetch(sampler2DMS(tex, tex_sampler), itex_coords, 0);
    color += texelFetch(sampler2DMS(tex, tex_sampler), itex_coords, 1);
    color += texelFetch(sampler2DMS(tex, tex_sampler), itex_coords, 2);
    color += texelFetch(sampler2DMS(tex, tex_sampler), itex_coords, 3);
    color *= 0.25;

    // Assign the resolved color to the output.
    f_color = color;
}
