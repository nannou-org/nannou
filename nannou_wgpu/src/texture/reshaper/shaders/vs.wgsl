struct VertexOutput {
    [[location(0)]] tex_coords: vec2<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> position1: vec2<f32>;
var<private> tex_coords: vec2<f32>;
var<private> gl_Position: vec4<f32>;

fn main1() {
    let _e3: vec2<f32> = position1;
    gl_Position = vec4<f32>(_e3, 0.0, 1.0);
    let _e7: vec2<f32> = position1;
    let _e14: vec2<f32> = position1;
    tex_coords = vec2<f32>(((_e7.x * 0.5) + 0.5), (1.0 - ((_e14.y * 0.5) + 0.5)));
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] position: vec2<f32>) -> VertexOutput {
    position1 = position;
    main1();
    let _e7: vec2<f32> = tex_coords;
    let _e9: vec4<f32> = gl_Position;
    return VertexOutput(_e7, _e9);
}
