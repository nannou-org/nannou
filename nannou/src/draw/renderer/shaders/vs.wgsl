[[block]]
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
    [[location(4)]] mat0: vec4<f32>,
    [[location(5)]] mat1: vec4<f32>,
    [[location(6)]] mat2: vec4<f32>,
    [[location(7)]] mat3: vec4<f32>,
) -> VertexOutput {
    let instance_transform: mat4x4<f32> = mat4x4<f32>(mat0, mat1, mat2, mat3);
    let out_pos: vec4<f32> = uniforms.proj * instance_transform * vec4<f32>(position, 1.0);
    return VertexOutput(color, tex_coords, mode, out_pos);
}
