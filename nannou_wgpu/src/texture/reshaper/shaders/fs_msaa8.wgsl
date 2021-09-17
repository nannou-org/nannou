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

    let _e26: vec2<i32> = textureDimensions(tex);
    tex_size = _e26;
    let _e28: i32 = tex_size[0u];
    let _e31: f32 = tex_coords1[0u];
    tex_x = i32((f32(_e28) * _e31));
    let _e35: i32 = tex_size[1u];
    let _e38: f32 = tex_coords1[1u];
    tex_y = i32((f32(_e35) * _e38));
    let _e41: i32 = tex_x;
    let _e42: i32 = tex_y;
    itex_coords = vec2<i32>(_e41, _e42);
    color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let _e44: vec2<i32> = itex_coords;
    let _e45: vec4<f32> = textureLoad(tex, _e44, 0);
    let _e46: vec4<f32> = color;
    color = (_e46 + _e45);
    let _e48: vec2<i32> = itex_coords;
    let _e49: vec4<f32> = textureLoad(tex, _e48, 1);
    let _e50: vec4<f32> = color;
    color = (_e50 + _e49);
    let _e52: vec2<i32> = itex_coords;
    let _e53: vec4<f32> = textureLoad(tex, _e52, 2);
    let _e54: vec4<f32> = color;
    color = (_e54 + _e53);
    let _e56: vec2<i32> = itex_coords;
    let _e57: vec4<f32> = textureLoad(tex, _e56, 3);
    let _e58: vec4<f32> = color;
    color = (_e58 + _e57);
    let _e60: vec2<i32> = itex_coords;
    let _e61: vec4<f32> = textureLoad(tex, _e60, 4);
    let _e62: vec4<f32> = color;
    color = (_e62 + _e61);
    let _e64: vec2<i32> = itex_coords;
    let _e65: vec4<f32> = textureLoad(tex, _e64, 5);
    let _e66: vec4<f32> = color;
    color = (_e66 + _e65);
    let _e68: vec2<i32> = itex_coords;
    let _e69: vec4<f32> = textureLoad(tex, _e68, 6);
    let _e70: vec4<f32> = color;
    color = (_e70 + _e69);
    let _e72: vec2<i32> = itex_coords;
    let _e73: vec4<f32> = textureLoad(tex, _e72, 7);
    let _e74: vec4<f32> = color;
    color = (_e74 + _e73);
    let _e76: vec4<f32> = color;
    color = (_e76 * 0.125);
    let _e78: vec4<f32> = color;
    f_color = _e78;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    tex_coords1 = tex_coords;
    main1();
    let _e3: vec4<f32> = f_color;
    return _e3;
}
