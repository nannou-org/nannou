[[group(0), binding(0)]]
var tex: texture_multisampled_2d<f32>;
[[group(0), binding(1)]]
var tex_sampler: sampler;
var<private> tex_coords1: vec2<f32>;
var<private> f_color: vec4<f32>;

fn main1() {
    var tex_size: vec2<i32>;
    var tex_x: i32;
    var tex_y: i32;
    var itex_coords: vec2<i32>;
    var color: vec4<f32>;

    let _e20: vec2<i32> = textureDimensions(tex);
    tex_size = _e20;
    let _e22: i32 = tex_size[0u];
    let _e25: f32 = tex_coords1[0u];
    tex_x = i32((f32(_e22) * _e25));
    let _e29: i32 = tex_size[1u];
    let _e32: f32 = tex_coords1[1u];
    tex_y = i32((f32(_e29) * _e32));
    let _e35: i32 = tex_x;
    let _e36: i32 = tex_y;
    itex_coords = vec2<i32>(_e35, _e36);
    color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let _e38: vec2<i32> = itex_coords;
    let _e39: vec4<f32> = textureLoad(tex, _e38, 0);
    let _e40: vec4<f32> = color;
    color = (_e40 + _e39);
    let _e42: vec2<i32> = itex_coords;
    let _e43: vec4<f32> = textureLoad(tex, _e42, 1);
    let _e44: vec4<f32> = color;
    color = (_e44 + _e43);
    let _e46: vec4<f32> = color;
    color = (_e46 * 0.5);
    let _e48: vec4<f32> = color;
    f_color = _e48;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    tex_coords1 = tex_coords;
    main1();
    let _e3: vec4<f32> = f_color;
    return _e3;
}
