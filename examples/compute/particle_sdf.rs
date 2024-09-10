use nannou::prelude::bevy_render::renderer::RenderDevice;
use nannou::prelude::bevy_render::storage::ShaderStorageBuffer;
use nannou::prelude::*;
use std::sync::Arc;

const WORKGROUP_SIZE: u32 = 64;

fn main() {
    nannou::app(model)
        .compute(compute)
        .update(update)
        .shader_model::<ShaderModel>()
        .run();
}

pub enum Shape {
    Circle,
    Square,
    Triangle,
}

struct Model {
    particles: Handle<ShaderStorageBuffer>,
    indirect_params: Handle<ShaderStorageBuffer>,
    circle_radius: f32,
    size: u32,
    buffer_size: u32,
    scaling_factor: u32,
}

impl Model {
    fn material(&self) -> ShaderModel {
        ShaderModel {
            particles: self.particles.clone(),
        }
    }
}

// This struct isn't used on the CPU side, but it's necessary to define the layout of the data
// to correctly size the buffer on the GPU side.
#[repr(C)]
#[derive(ShaderType)]
struct Particle {
    position: Vec2,
    original_position: Vec2,
    velocity: Vec2,
    energy: f32,
    _pad: f32,
    color: Vec4,
}

// The draw indirect args struct is used to pass the number of instances to draw to the GPU
// The compute shader will write the number of particles to draw to `instance_count`
#[repr(C)]
#[derive(ShaderType)]
struct DrawIndirectArgs {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

// Our compute shader has two states, `Init` and `Update`
// The `Init` state is used to initialize the particles
// The `Update` state is used to run the simulation
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
enum State {
    Init(u32),
    Update(u32),
}

impl Default for State {
    fn default() -> Self {
        State::Init(1)
    }
}

// Compute model, defining uniforms and storage buffers that will be passed to the compute shader
#[derive(AsBindGroup, Clone)]
struct ComputeModel {
    #[storage(0, visibility(compute))]
    particles: Handle<ShaderStorageBuffer>,
    #[uniform(1)]
    circle_center: Vec4,
    #[uniform(2)]
    circle_radius: f32,
    #[uniform(3)]
    scaling_factor: u32,
    #[uniform(4)]
    resolution: UVec2,
    #[storage(5, visibility(compute))]
    indirect_params: Handle<ShaderStorageBuffer>,
}

impl Compute for ComputeModel {
    type State = State;

    fn shader() -> ShaderRef {
        "shaders/particle_sdf_compute.wgsl".into()
    }

    fn entry(state: &Self::State) -> &'static str {
        match state {
            State::Init(_) => "init",
            State::Update(_) => "update",
        }
    }

    fn dispatch_size(state: &Self::State) -> (u32, u32, u32) {
        let size = match state {
            State::Init(size) => size,
            State::Update(size) => size,
        };

        ((size + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE, 1, 1)
    }
}

// Our shader model that will be used to render the particles
#[shader_model(
    fragment = "shaders/particle_sdf_material.wgsl",
    vertex = "shaders/particle_sdf_material.wgsl"
)]
struct ShaderModel {
    #[storage(0, read_only, visibility(vertex))]
    particles: Handle<ShaderStorageBuffer>,
}

fn model(app: &App) -> Model {
    let _window_id = app
        .new_window()
        .primary()
        .size(1024, 768)
        .view(view)
        .build();

    // Create a buffer to store the particles.
    let size = 1; // This will be updated in the `update` function
    let particles = create_particle_buffer(app, size);

    // Create a buffer store our indirect draw params
    let indirect_params = create_indirect_params_buffer(app, size);

    Model {
        particles,
        indirect_params,
        circle_radius: 0.5,
        size,
        buffer_size: size,
        scaling_factor: 50,
    }
}

// Controls:
// - Arrow keys to change the circle radius
// - Left and right arrow keys to change the scaling factor (i.e. density)
fn update(app: &App, model: &mut Model) {
    if app.keys().pressed(KeyCode::ArrowUp) {
        model.circle_radius += 0.01;
    }
    if app.keys().pressed(KeyCode::ArrowDown) {
        model.circle_radius -= 0.01;
    }
    if app.keys().pressed(KeyCode::ArrowRight) {
        model.scaling_factor += 1;
    }
    if app.keys().pressed(KeyCode::ArrowLeft) {
        // Don't let the scaling factor go below 20
        model.scaling_factor = model.scaling_factor.saturating_sub(1).max(20);
    }


    let pixels = app.main_window().size_pixels().element_product();
    let new_size = pixels / model.scaling_factor.clamp(20, 1000);

    // Resize the buffer if necessary
    if new_size > model.buffer_size {
        model.particles = create_particle_buffer(app, new_size);
        model.buffer_size = new_size;
    }

    model.size = new_size;
}

fn compute(
    app: &App,
    model: &Model,
    previous_state: State,
    _view: Entity,
) -> (State, ComputeModel) {
    let window = app.main_window();
    let window_rect = window.rect();

    let mouse_pos = app.mouse();
    let mouse_norm = Vec2::new(
        mouse_pos.x / window_rect.w() * 2.0,
        mouse_pos.y / window_rect.h() * 2.0,
    );

    let compute_model = ComputeModel {
        particles: model.particles.clone(),
        circle_center: mouse_norm.extend(1.0).extend(0.0),
        circle_radius: model.circle_radius,
        scaling_factor: model.scaling_factor,
        resolution: window.size_pixels(),
        indirect_params: model.indirect_params.clone(),
    };

    // If the size has changed, we need to re-initialize the particles
    // Otherwise, we can just update them
    match previous_state {
        State::Init(size) => {
            // Even if we are in the `Init` state, we still need to update the size of the buffer
            // in case the window has been resized
            if size != model.size {
                (State::Init(model.size), compute_model)
            } else {
                (State::Update(model.size), compute_model)
            }
        }
        State::Update(size) => {
            if size != model.size {
                (State::Init(model.size), compute_model)
            } else {
                (State::Update(model.size), compute_model)
            }
        }
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(GRAY);

    let draw = draw.material(model.material());
    draw.indirect()
        .primitive(draw.rect().w_h(2.0, 2.0))
        .buffer(model.indirect_params.clone());
}

fn create_particle_buffer(app: &App, size: u32) -> Handle<ShaderStorageBuffer> {
    let particle_size = Particle::min_size().get() as usize;
    let mut particles = ShaderStorageBuffer::with_size(
        size as usize * particle_size * 2,
        RenderAssetUsages::RENDER_WORLD,
    );
    particles.buffer_description.label = Some("particles");
    particles.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::VERTEX;
    let particles = app.assets_mut().add(particles);
    particles
}

fn create_indirect_params_buffer(app: &App, size: u32) -> Handle<ShaderStorageBuffer> {
    let mut indirect_params = ShaderStorageBuffer::from(DrawIndirectArgs {
        index_count: 6, // Hardcoded for now, 2 triangles
        instance_count: size,
        first_index: 0,
        base_vertex: 0,
        first_instance: 0,
    });
    indirect_params.buffer_description.label = Some("indirect_params");
    indirect_params.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::INDIRECT;
    let indirect_params = app.assets_mut().add(indirect_params);
    indirect_params
}
