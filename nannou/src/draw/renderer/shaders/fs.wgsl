struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(1) @binding(0)
var text_sampler: sampler;
@group(1) @binding(1)
var text: texture_2d<f32>;
@group(2) @binding(0)
var tex_sampler: sampler;
@group(2) @binding(1)
var tex: texture_2d<f32>;

@fragment
fn main(
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) mode: u32,
) -> FragmentOutput {
    let tex_color: vec4<f32> = textureSample(tex, tex_sampler, tex_coords);
    let text_color: vec4<f32> = textureSample(text, text_sampler, tex_coords);
    let text_alpha: f32 = text_color.x;
    var out_color: vec4<f32>;
    if (mode == u32(0)) {
        out_color = color;
    } else {
        if (mode == u32(1)) {
            out_color = tex_color;
        } else {
            if (mode == u32(2)) {
                out_color = vec4<f32>(color.xyz, color.w * text_alpha);
            } else {
                out_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }
        }
    }
    return FragmentOutput(out_color);
}
