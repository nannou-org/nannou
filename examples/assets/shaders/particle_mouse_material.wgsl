#import bevy_pbr::{
    forward_io::{Vertex}
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world}
    view_transformations::{position_world_to_clip}
}

struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec4<f32>,
};

@group(2) @binding(0) var<storage, read> particles: array<Particle>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let particle = particles[vertex.instance_index];
    var out: VertexOutput;

    if (vertex.instance_index == 100) {
        out.clip_position = vec4(particle.position, 0.0, 1.0);
        out.color = vec4(1.0, 0.0, 0.0, 1.0);  // Red color for visibility
    } else {
        out.clip_position = mesh_position_local_to_clip(
            get_world_from_local(vertex.instance_index),
            vec4(vertex.position, 1.0)
        ) + vec4(particle.position, 0.0, 0.0);
        out.color = particle.color;
    }

    return out;
}



@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}