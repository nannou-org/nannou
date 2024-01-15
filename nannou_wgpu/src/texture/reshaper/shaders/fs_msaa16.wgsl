struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
};

@group(0) @binding(0)
var tex: texture_multisampled_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn main(
    @location(0) tex_coords: vec2<f32>,
) -> FragmentOutput {
    // Get the integer tex coordinates.
    let tex_size: vec2<u32> = textureDimensions(tex);
    let tex_x: i32 = i32(f32(tex_size.x) * tex_coords.x);
    let tex_y: i32 = i32(f32(tex_size.y) * tex_coords.y);;
    let itex_coords: vec2<i32> = vec2<i32>(tex_x, tex_y);

    // Manually unroll the resolve. The less conditions the better!
    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    color = color + textureLoad(tex, itex_coords, 0);
    color = color + textureLoad(tex, itex_coords, 1);
    color = color + textureLoad(tex, itex_coords, 2);
    color = color + textureLoad(tex, itex_coords, 3);
    color = color + textureLoad(tex, itex_coords, 4);
    color = color + textureLoad(tex, itex_coords, 5);
    color = color + textureLoad(tex, itex_coords, 6);
    color = color + textureLoad(tex, itex_coords, 7);
    color = color + textureLoad(tex, itex_coords, 8);
    color = color + textureLoad(tex, itex_coords, 9);
    color = color + textureLoad(tex, itex_coords, 10);
    color = color + textureLoad(tex, itex_coords, 11);
    color = color + textureLoad(tex, itex_coords, 12);
    color = color + textureLoad(tex, itex_coords, 13);
    color = color + textureLoad(tex, itex_coords, 14);
    color = color + textureLoad(tex, itex_coords, 15);
    color = color * 0.0625;

    return FragmentOutput(color);
}
