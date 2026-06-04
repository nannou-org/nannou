struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> mouse: vec2<f32>;
@group(0) @binding(2) var<uniform> attract_strength: f32;
@group(0) @binding(3) var<uniform> particle_count: u32;

fn random(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2(12.9898, 78.233))) * 43758.5453);
}

@compute @workgroup_size(64)
fn init(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= particle_count) {
        return;
    }

    var particle: Particle;

    // Initialize position randomly within clip space (-1 to 1)
    particle.position = vec2<f32>(
        random(vec2<f32>(f32(index), 0.0)) * 2.0 - 1.0,
        random(vec2<f32>(0.0, f32(index))) * 2.0 - 1.0
    );

    // Initialize velocity (in clip space, so use smaller values)
    particle.velocity = vec2<f32>(
        (random(vec2<f32>(f32(index), 1.0)) - 0.5) * 0.002,
        (random(vec2<f32>(1.0, f32(index))) - 0.5) * 0.002
    );

    particle.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    particles[index] = particle;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= particle_count) {
        return;
    }

    var particle = particles[index];

    // Update particle position
    particle.position = particle.position + particle.velocity;

    // Attract particles to mouse
    let to_mouse = mouse - particle.position;
    particle.velocity = particle.velocity + (normalize(to_mouse) * 0.00001 * attract_strength);

    // Bounce off screen edges
    if (particle.position.x < -1.0 || particle.position.x > 1.0) {
        particle.velocity.x = -particle.velocity.x;
    }
    if (particle.position.y < -1.0 || particle.position.y > 1.0) {
        particle.velocity.y = -particle.velocity.y;
    }

    // Keep particles within bounds
    particle.position = clamp(particle.position, vec2(-1.0), vec2(1.0));
    // Limit velocity to prevent particles from becoming too energetic
    particle.velocity = clamp(particle.velocity, vec2(-0.01), vec2(0.01));

    // Update color based on velocity
    particle.color = vec4<f32>(abs(particle.velocity) * 100.0, 1.0, 1.0);
    if (index == 0) {
        particle.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);  // Red for first particle
    }

    particles[index] = particle;
}