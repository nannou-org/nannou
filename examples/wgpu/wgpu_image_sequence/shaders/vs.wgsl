struct VertexOutput {
    [[location(0)]] tex_coords: vec2<f32>;
    [[builtin(position)]] out_pos: vec4<f32>;
};

[[stage(vertex)]]
fn main([[location(0)]] pos: vec2<f32>) -> VertexOutput {
    let tex_coords: vec2<f32> = vec2<f32>(pos.x * 0.5 + 0.5, 1.0 - (pos.y * 0.5 + 0.5));
    let out_pos: vec4<f32> = vec4<f32>(pos, 0.0, 1.0);
    return VertexOutput(tex_coords, out_pos);
}
