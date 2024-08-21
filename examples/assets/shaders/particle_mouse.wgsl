struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> mouse: vec2<f32>;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let particle = particles[vertex_index];

    var out: VertexOutput;
    out.clip_position = vec4<f32>(particle.position, 0.0, 1.0);
    out.color = particle.color;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

fn random(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2(12.9898, 78.233))) * 43758.5453);
}

@compute @workgroup_size(64)
fn init(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    var particle: Particle;

    // Initialize position randomly within the window
    particle.position = vec2<f32>(
        (random(vec2<f32>(f32(index), 0.0)) - 0.5) * resolution.x,
        (random(vec2<f32>(0.0, f32(index))) - 0.5) * resolution.y
    );

    // Initialize velocity randomly
    particle.velocity = vec2<f32>(
        (random(vec2<f32>(f32(index), 1.0)) - 0.5) * 0.1,
        (random(vec2<f32>(1.0, f32(index))) - 0.5) * 0.1
    );

    // Initialize color randomly
    particle.color = vec4<f32>(
        random(vec2<f32>(f32(index), 2.0)),
        random(vec2<f32>(2.0, f32(index))),
        random(vec2<f32>(f32(index), 3.0)),
        1.0
    );

    particles[index] = particle;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    var particle = particles[index];

    // Update particle position
    particle.position = particle.position + particle.velocity;

    // Attract particles to mouse
    let to_mouse = mouse - particle.position;
    particle.velocity = particle.velocity + normalize(to_mouse) * 0.0001;

    // Bounce off screen edges
    if (particle.position.x < -1.0 || particle.position.x > 1.0) {
        particle.velocity.x = -particle.velocity.x;
    }
    if (particle.position.y < -1.0 || particle.position.y > 1.0) {
        particle.velocity.y = -particle.velocity.y;
    }

    // Update color based on velocity
    particle.color = vec4<f32>(abs(particle.velocity), 1.0, 1.0);

    particles[index] = particle;
}
