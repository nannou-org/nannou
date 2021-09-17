[[block]]
struct Data {
    sample_count: u32;
};

[[group(0), binding(0)]]
var tex: texture_multisampled_2d<f32>;
[[group(0), binding(1)]]
var tex_sampler: sampler;
var<private> tex_coords1: vec2<f32>;
[[group(0), binding(2)]]
var<uniform> uniforms: Data;
var<private> f_color: vec4<f32>;

fn main1() {
    var tex_size: vec2<i32>;
    var tex_x: i32;
    var tex_y: i32;
    var itex_coords: vec2<i32>;
    var color: vec4<f32>;
    var i: i32;

    let _e21: vec2<i32> = textureDimensions(tex);
    tex_size = _e21;
    let _e23: i32 = tex_size[0u];
    let _e26: f32 = tex_coords1[0u];
    tex_x = i32((f32(_e23) * _e26));
    let _e30: i32 = tex_size[1u];
    let _e33: f32 = tex_coords1[1u];
    tex_y = i32((f32(_e30) * _e33));
    let _e36: i32 = tex_x;
    let _e37: i32 = tex_y;
    itex_coords = vec2<i32>(_e36, _e37);
    color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    i = 0;
    loop {
        let _e39: i32 = i;
        let _e42: u32 = uniforms.sample_count;
        if ((bitcast<u32>(_e39) < _e42)) {
            let _e44: vec2<i32> = itex_coords;
            let _e45: i32 = i;
            let _e46: vec4<f32> = textureLoad(tex, _e44, _e45);
            let _e47: vec4<f32> = color;
            color = (_e47 + _e46);
            continue;
        } else {
            break;
        }
        continuing {
            let _e49: i32 = i;
            i = (_e49 + 1);
        }
    }
    let _e52: u32 = uniforms.sample_count;
    let _e54: vec4<f32> = color;
    color = (_e54 / vec4<f32>(f32(_e52)));
    let _e57: vec4<f32> = color;
    f_color = _e57;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> [[location(0)]] vec4<f32> {
    tex_coords1 = tex_coords;
    main1();
    let _e3: vec4<f32> = f_color;
    return _e3;
}
