[[stage(vertex)]]
fn main([[location(0)]] pos: vec2<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}
