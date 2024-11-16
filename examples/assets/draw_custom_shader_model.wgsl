#import bevy_pbr::forward_io::VertexOutput

struct CustomShaderModel {
    color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> shader_model: CustomShaderModel;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return shader_model.color;
}
