struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
};

[[block]]
struct Data {
    sample_count: u32;
};

@group(0) @binding(0)
var tex: texture_multisampled_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;
[[group(0), binding(2)]]
var<uniform> uniforms: Data;

@fragment
fn main(
    @location(0) tex_coords: vec2<f32>,
) -> FragmentOutput {
    // Get the integer tex coordinates.
    let tex_size: vec2<i32> = textureDimensions(tex);
    let tex_x: i32 = i32(f32(tex_size.x) * tex_coords.x);
    let tex_y: i32 = i32(f32(tex_size.y) * tex_coords.y);;
    let itex_coords: vec2<i32> = vec2<i32>(tex_x, tex_y);

    // Perform the resolve.
    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var i: i32 = 0;
    loop {
        if (u32(i) < uniforms.sample_count) {
            color = color + textureLoad(tex, itex_coords, i);
            continue;
        } else {
            break;
        }
        continuing {
            i = i + 1;
        }
    }
    color = color / vec4<f32>(f32(uniforms.sample_count));

    return FragmentOutput(color);
}
