struct FragmentOutput {
    [[location(0)]] f_color: vec4<f32>;
};

[[group(1), binding(0)]]
var text_sampler: sampler;
[[group(1), binding(1)]]
var text: texture_2d<f32>;
[[group(2), binding(0)]]
var tex_sampler: sampler;
[[group(2), binding(1)]]
var tex: texture_2d<f32>;
var<private> v_color1: vec4<f32>;
var<private> v_tex_coords1: vec2<f32>;
var<private> v_mode1: u32;
var<private> f_color: vec4<f32>;

fn main1() {
    var tex_color: vec4<f32>;
    var text_alpha: f32;

    let _e9: vec2<f32> = v_tex_coords1;
    let _e10: vec4<f32> = textureSample(tex, tex_sampler, _e9);
    tex_color = _e10;
    let _e13: vec2<f32> = v_tex_coords1;
    let _e14: vec4<f32> = textureSample(text, text_sampler, _e13);
    text_alpha = _e14.x;
    let _e17: u32 = v_mode1;
    if ((_e17 == u32(0))) {
        {
            let _e21: vec4<f32> = v_color1;
            f_color = _e21;
            return;
        }
    } else {
        let _e22: u32 = v_mode1;
        if ((_e22 == u32(1))) {
            {
                let _e26: vec4<f32> = tex_color;
                f_color = _e26;
                return;
            }
        } else {
            let _e27: u32 = v_mode1;
            if ((_e27 == u32(2))) {
                {
                    let _e31: vec4<f32> = v_color1;
                    let _e33: vec4<f32> = v_color1;
                    let _e35: f32 = text_alpha;
                    f_color = vec4<f32>(_e31.xyz, (_e33.w * _e35));
                    return;
                }
            } else {
                {
                    f_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
                    return;
                }
            }
        }
    }
}

[[stage(fragment)]]
fn main([[location(0)]] v_color: vec4<f32>, [[location(1)]] v_tex_coords: vec2<f32>, [[location(2)]] v_mode: u32) -> FragmentOutput {
    v_color1 = v_color;
    v_tex_coords1 = v_tex_coords;
    v_mode1 = v_mode;
    main1();
    let _e23: vec4<f32> = f_color;
    return FragmentOutput(_e23);
}
