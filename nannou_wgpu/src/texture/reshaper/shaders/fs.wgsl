struct FragmentOutput {
    [[location(0)]] f_color: vec4<f32>;
};

var<private> tex_coords1: vec2<f32>;
var<private> f_color: vec4<f32>;
[[group(0), binding(0)]]
var tex: texture_2d<f32>;
[[group(0), binding(1)]]
var tex_sampler: sampler;

fn main1() {
    let _e5: vec2<f32> = tex_coords1;
    let _e6: vec4<f32> = textureSample(tex, tex_sampler, _e5);
    f_color = _e6;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> FragmentOutput {
    tex_coords1 = tex_coords;
    main1();
    let _e11: vec4<f32> = f_color;
    return FragmentOutput(_e11);
}
