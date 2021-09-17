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

    let _e34: vec2<i32> = textureDimensions(tex);
    tex_size = _e34;
    let _e36: i32 = tex_size[0u];
    let _e39: f32 = tex_coords1[0u];
    tex_x = i32((f32(_e36) * _e39));
    let _e43: i32 = tex_size[1u];
    let _e46: f32 = tex_coords1[1u];
    tex_y = i32((f32(_e43) * _e46));
    let _e49: i32 = tex_x;
    let _e50: i32 = tex_y;
    itex_coords = vec2<i32>(_e49, _e50);
    color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let _e52: vec2<i32> = itex_coords;
    let _e53: vec4<f32> = textureLoad(tex, _e52, 0);
    let _e54: vec4<f32> = color;
    color = (_e54 + _e53);
    let _e56: vec2<i32> = itex_coords;
    let _e57: vec4<f32> = textureLoad(tex, _e56, 1);
    let _e58: vec4<f32> = color;
    color = (_e58 + _e57);
    let _e60: vec2<i32> = itex_coords;
    let _e61: vec4<f32> = textureLoad(tex, _e60, 2);
    let _e62: vec4<f32> = color;
    color = (_e62 + _e61);
    let _e64: vec2<i32> = itex_coords;
    let _e65: vec4<f32> = textureLoad(tex, _e64, 3);
    let _e66: vec4<f32> = color;
    color = (_e66 + _e65);
    let _e68: vec2<i32> = itex_coords;
    let _e69: vec4<f32> = textureLoad(tex, _e68, 4);
    let _e70: vec4<f32> = color;
    color = (_e70 + _e69);
    let _e72: vec2<i32> = itex_coords;
    let _e73: vec4<f32> = textureLoad(tex, _e72, 5);
    let _e74: vec4<f32> = color;
    color = (_e74 + _e73);
    let _e76: vec2<i32> = itex_coords;
    let _e77: vec4<f32> = textureLoad(tex, _e76, 6);
    let _e78: vec4<f32> = color;
    color = (_e78 + _e77);
    let _e80: vec2<i32> = itex_coords;
    let _e81: vec4<f32> = textureLoad(tex, _e80, 7);
    let _e82: vec4<f32> = color;
    color = (_e82 + _e81);
    let _e84: vec2<i32> = itex_coords;
    let _e85: vec4<f32> = textureLoad(tex, _e84, 8);
    let _e86: vec4<f32> = color;
    color = (_e86 + _e85);
    let _e88: vec2<i32> = itex_coords;
    let _e89: vec4<f32> = textureLoad(tex, _e88, 9);
    let _e90: vec4<f32> = color;
    color = (_e90 + _e89);
    let _e92: vec2<i32> = itex_coords;
    let _e93: vec4<f32> = textureLoad(tex, _e92, 10);
    let _e94: vec4<f32> = color;
    color = (_e94 + _e93);
    let _e96: vec2<i32> = itex_coords;
    let _e97: vec4<f32> = textureLoad(tex, _e96, 11);
    let _e98: vec4<f32> = color;
    color = (_e98 + _e97);
    let _e100: vec2<i32> = itex_coords;
    let _e101: vec4<f32> = textureLoad(tex, _e100, 12);
    let _e102: vec4<f32> = color;
    color = (_e102 + _e101);
    let _e104: vec2<i32> = itex_coords;
    let _e105: vec4<f32> = textureLoad(tex, _e104, 13);
    let _e106: vec4<f32> = color;
    color = (_e106 + _e105);
    let _e108: vec2<i32> = itex_coords;
    let _e109: vec4<f32> = textureLoad(tex, _e108, 14);
    let _e110: vec4<f32> = color;
    color = (_e110 + _e109);
    let _e112: vec2<i32> = itex_coords;
    let _e113: vec4<f32> = textureLoad(tex, _e112, 15);
    let _e114: vec4<f32> = color;
    color = (_e114 + _e113);
    let _e116: vec4<f32> = color;
    color = (_e116 * 0.0625);
    let _e118: vec4<f32> = color;
    f_color = _e118;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    tex_coords1 = tex_coords;
    main1();
    let _e3: vec4<f32> = f_color;
    return _e3;
}
