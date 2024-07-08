#import bevy_pbr::{
    mesh_view_bindings::view,
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let resolution = view.viewport.zw;
    let time = globals.time;
    var uv = mesh.uv * resolution;
    let cp = -1.0 + 2.0 * uv / resolution;
    let cl = length(cp);
    uv = uv / resolution + (cp / cl) * cos(cl * 12.0 - time * 1.0) * 0.02;
    let clamped_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
    let col = textureSample(texture, texture_sampler, clamped_uv).xyz;
    return vec4<f32>(col, 1.0);
}
