#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(
    vertex: VertexOutput,
) -> @location(0) vec4<f32> {
    // Filter out the blue channel of the vertex color
    return vec4(vertex.color.xy, 0.0, 1.0);
}
