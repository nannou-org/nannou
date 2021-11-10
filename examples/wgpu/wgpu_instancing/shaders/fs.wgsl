struct FragmentOutput {
    [[location(0)]] f_color: vec4<f32>;
};

[[stage(fragment)]]
fn main(
    [[location(0)]] normal: vec3<f32>,
    [[location(1)]] color: vec3<f32>,
) -> FragmentOutput {
    let light: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
    let brightness: f32 = dot(normalize(normal), normalize(light));
    let dark_color: vec3<f32> = vec3<f32>(0.1, 0.1, 0.1) * color;
    let out_color = vec4<f32>(mix(dark_color, color, vec3<f32>(brightness)), 1.0);
    return FragmentOutput(out_color);
}
