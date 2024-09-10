struct Particle {
    position: vec2<f32>,
    original_position: vec2<f32>,
    velocity: vec2<f32>,
    energy: f32,
    color: vec4<f32>,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> sphere_center: vec4<f32>;
@group(0) @binding(2) var<uniform> sphere_radius: f32;
@group(0) @binding(3) var<uniform> particle_count: u32;
@group(0) @binding(4) var<uniform> resolution: vec2<u32>;

fn sdf_sphere(point: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(point - center) - radius;
}

fn get_ray_direction(uv: vec2<f32>) -> vec3<f32> {
    let aspect = f32(resolution.x) / f32(resolution.y);
    return normalize(vec3((uv.x * 2.0 - 1.0) * aspect, (uv.y * 2.0 - 1.0), -1.0));
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>, max_steps: u32, max_dist: f32) -> f32 {
    var total_distance = 0.0;
    for (var i: u32 = 0u; i < max_steps; i++) {
        let p = ro + total_distance * rd;
        let distance = sdf_sphere(p, sphere_center.xyz, sphere_radius);
        if distance < 0.001 {
            return total_distance;
        }
        total_distance += distance;
        if total_distance > max_dist {
            break;
        }
    }
    return -1.0;
}

fn closest_point_on_sphere(point: vec2<f32>) -> vec3<f32> {
    let aspect = f32(resolution.x) / f32(resolution.y);
    // Adjust for the new coordinate system
    let point3d = vec3(point.x * aspect, point.y, 0.0);
    let to_sphere = normalize(point3d - sphere_center.xyz);
    let surface_point = sphere_center.xyz + to_sphere * sphere_radius;
    return vec3(surface_point.x / surface_point.z / aspect, surface_point.y / surface_point.z, surface_point.z);
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= particle_count) {
        return;
    }

    var particle = particles[index];

    // Convert particle position to UV coordinates
    let uv = (particle.original_position + 1.0) * 0.5;

    // Get ray direction for this particle
    let ray_direction = get_ray_direction(uv);

    // Set camera position
    let camera_position = vec3(0.0, 0.0, 2.0);

    // Perform raymarching
    let hit_distance = raymarch(camera_position, ray_direction, 100u, 3.0);

    let max_speed = 0.01;
    let acceleration = 0.0005;
    let damping = 0.95;  // Damping factor to reduce oscillations

    let sphere_point = closest_point_on_sphere(particle.original_position);
    let target_position = sphere_point.xy;

    if (hit_distance > 0.0) {
        let to_target = target_position - particle.position;
        let distance_to_target = length(to_target);

        if (distance_to_target > 0.001) {
            let direction = normalize(to_target);
            particle.velocity += direction * acceleration;

            // Apply damping
            particle.velocity *= damping;

            // Cap speed
            let speed = length(particle.velocity);
            if (speed > max_speed) {
                particle.velocity = normalize(particle.velocity) * max_speed;
            }
        } else {
            // Very close to target, stop movement
            particle.velocity = vec2(0.0, 0.0);
            particle.position = target_position;
        }

        particle.energy = min(particle.energy + 0.1, 1.0);
    } else {
        // The ray didn't hit, move back to the original position
        let to_original = particle.original_position - particle.position;
        let distance_to_original = length(to_original);

        if (distance_to_original > 0.001) {
            let direction = normalize(to_original);
            particle.velocity += direction * acceleration;

            // Apply damping
            particle.velocity *= damping;

            // Cap speed
            let speed = length(particle.velocity);
            if (speed > max_speed) {
                particle.velocity = normalize(particle.velocity) * max_speed;
            }
        } else {
            // Very close to original position, stop movement
            particle.velocity = vec2(0.0, 0.0);
            particle.position = particle.original_position;
        }

        particle.energy = max(particle.energy - 0.05, 0.0);
    }

    // Update position
    particle.position += particle.velocity;

    // Calculate depth based on current particle position relative to sphere center
    let aspect = f32(resolution.x) / f32(resolution.y);
    let p = closest_point_on_sphere(particle.position);
    let particle_3d = vec3(particle.position.x * aspect, particle.position.y, p.z);

    // Calculate the vector from sphere center to the particle on the sphere surface
    let to_particle = particle_3d - sphere_center.xyz;

    // Calculate the view direction (assuming camera is looking along negative z-axis)
    let view_direction = vec3(0.0, 0.0, -1.0);

    // Calculate the angle between to_particle and view_direction
    let cos_angle = dot(normalize(to_particle), -view_direction);

    // Calculate depth based on this angle
    // This will be 1 at the center (cos_angle = 1) and 0 at the edges (cos_angle = 0)
    let depth = cos_angle;

    // Optionally, apply a non-linear function to emphasize the effect
    let emphasized_depth = pow(depth, 2.0);  // Adjust power as needed

    // Update color based on energy and depth
    let unenergized_color = vec3(0.1, 0.1, 0.1);
    let energized_color = vec3(emphasized_depth, 0.0, 0.0);
    let color = mix(unenergized_color, energized_color, particle.energy);
    particle.color = vec4(color, 1.0);

    particles[index] = particle;
}


@compute @workgroup_size(64)
fn init(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= particle_count) {
        return;
    }

    var particle: Particle;

    // Calculate aspect ratio
    let aspect_ratio = f32(resolution.x) / f32(resolution.y);

    // Determine grid dimensions
    let grid_width = sqrt(f32(particle_count) * aspect_ratio);
    let grid_height = f32(particle_count) / grid_width;

    // Calculate particle position in grid
    let grid_x = f32(index % u32(grid_width)) / (grid_width - 1.0);
    let grid_y = floor(f32(index) / grid_width) / (grid_height - 1.0);

    // Map grid position to screen space (-1 to 1 for both axes)
    let screen_x = grid_x * 2.0 - 1.0;
    let screen_y = grid_y * 2.0 - 1.0;

    particle.position = vec2(screen_x, screen_y);
    particle.original_position = particle.position;
    particle.energy = 0.0;
    particle.velocity = vec2(0.0, 0.0);
    particle.color = vec4(0.1, 0.1, 0.1, 1.0);

    particles[index] = particle;
}