struct Data {
    proj: mat4x4<f32>;
};

struct VertexOutput {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] mode: u32;
    [[builtin(position)]] pos: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Data;

[[stage(vertex)]]
fn main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] color: vec4<f32>,
    [[location(2)]] tex_coords: vec2<f32>,
    [[location(3)]] mode: u32,
) -> VertexOutput {
    let out_pos: vec4<f32> = uniforms.proj * vec4<f32>(position, 1.0);
    return VertexOutput(color, tex_coords, mode, out_pos);
}
