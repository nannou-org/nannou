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

    let _e22: vec2<i32> = textureDimensions(tex);
    tex_size = _e22;
    let _e24: i32 = tex_size[0u];
    let _e27: f32 = tex_coords1[0u];
    tex_x = i32((f32(_e24) * _e27));
    let _e31: i32 = tex_size[1u];
    let _e34: f32 = tex_coords1[1u];
    tex_y = i32((f32(_e31) * _e34));
    let _e37: i32 = tex_x;
    let _e38: i32 = tex_y;
    itex_coords = vec2<i32>(_e37, _e38);
    color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let _e40: vec2<i32> = itex_coords;
    let _e41: vec4<f32> = textureLoad(tex, _e40, 0);
    let _e42: vec4<f32> = color;
    color = (_e42 + _e41);
    let _e44: vec2<i32> = itex_coords;
    let _e45: vec4<f32> = textureLoad(tex, _e44, 1);
    let _e46: vec4<f32> = color;
    color = (_e46 + _e45);
    let _e48: vec2<i32> = itex_coords;
    let _e49: vec4<f32> = textureLoad(tex, _e48, 2);
    let _e50: vec4<f32> = color;
    color = (_e50 + _e49);
    let _e52: vec2<i32> = itex_coords;
    let _e53: vec4<f32> = textureLoad(tex, _e52, 3);
    let _e54: vec4<f32> = color;
    color = (_e54 + _e53);
    let _e56: vec4<f32> = color;
    color = (_e56 * 0.25);
    let _e58: vec4<f32> = color;
    f_color = _e58;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    tex_coords1 = tex_coords;
    main1();
    let _e3: vec4<f32> = f_color;
    return _e3;
}
