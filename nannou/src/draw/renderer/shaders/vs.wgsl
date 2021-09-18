[[block]]
struct Data {
    proj: mat4x4<f32>;
};

struct VertexOutput {
    [[location(0)]] v_color: vec4<f32>;
    [[location(1)]] v_tex_coords: vec2<f32>;
    [[location(2)]] v_mode: u32;
    [[builtin(position)]] member: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Data;
var<private> position1: vec3<f32>;
var<private> color1: vec4<f32>;
var<private> tex_coords1: vec2<f32>;
var<private> mode1: u32;
var<private> v_color: vec4<f32>;
var<private> v_tex_coords: vec2<f32>;
var<private> v_mode: u32;
var<private> gl_Position: vec4<f32>;

fn main1() {
    let _e11: Data = uniforms;
    let _e13: vec3<f32> = position1;
    gl_Position = (_e11.proj * vec4<f32>(_e13, 1.0));
    let _e17: vec4<f32> = color1;
    v_color = _e17;
    let _e18: vec2<f32> = tex_coords1;
    v_tex_coords = _e18;
    let _e19: u32 = mode1;
    v_mode = _e19;
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] position: vec3<f32>, [[location(1)]] color: vec4<f32>, [[location(2)]] tex_coords: vec2<f32>, [[location(3)]] mode: u32) -> VertexOutput {
    position1 = position;
    color1 = color;
    tex_coords1 = tex_coords;
    mode1 = mode;
    main1();
    let _e26: vec4<f32> = v_color;
    let _e28: vec2<f32> = v_tex_coords;
    let _e30: u32 = v_mode;
    let _e32: vec4<f32> = gl_Position;
    return VertexOutput(_e26, _e28, _e30, _e32);
}
