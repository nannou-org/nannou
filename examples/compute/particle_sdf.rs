use nannou::prelude::bevy_render::renderer::RenderDevice;
use nannou::prelude::bevy_render::storage::ShaderStorageBuffer;
use nannou::prelude::*;
use std::sync::Arc;

const NUM_PARTICLES: u32 = 100000;
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
}

impl Model {
    fn material(&self) -> ShaderModel {
        ShaderModel {
            particles: self.particles.clone(),
        }
    }
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: Vec4,
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawIndirectArgs {
    /// The number of indices to draw.
    pub index_count: u32,
    /// The number of instances to draw.
    pub instance_count: u32,
    /// The first index within the index buffer.
    pub first_index: u32,
    /// The value added to the vertex index before indexing into the vertex buffer.
    pub base_vertex: i32,
    /// The instance ID of the first instance to draw.
    ///
    /// Has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`](crate::Features::INDIRECT_FIRST_INSTANCE) is enabled.
    pub first_instance: u32,
}

#[derive(Default, Debug, Eq, PartialEq, Hash, Clone)]
enum State {
    #[default]
    Init,
    Update,
}

#[derive(AsBindGroup, Clone)]
struct ComputeModel {
    #[storage(0, visibility(compute))]
    particles: Handle<ShaderStorageBuffer>,
    #[uniform(1)]
    circle_center: Vec4,
    #[uniform(2)]
    circle_radius: f32,
    #[uniform(3)]
    particle_count: u32,
    #[uniform(4)]
    resolution: UVec2,
}

impl Compute for ComputeModel {
    type State = State;

    fn shader() -> ShaderRef {
        "shaders/particle_sdf_compute.wgsl".into()
    }

    fn entry(state: &Self::State) -> &'static str {
        match state {
            State::Init => "init",
            State::Update => "update",
        }
    }

    fn dispatch_size(_state: &Self::State) -> (u32, u32, u32) {
        ((NUM_PARTICLES + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE, 1, 1)
    }
}

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
    let particle_size = Particle::min_size().get() as usize;
    let mut particles = ShaderStorageBuffer::with_size(
        NUM_PARTICLES as usize * particle_size * 2,
        RenderAssetUsages::RENDER_WORLD,
    );
    particles.buffer_description.label = Some("particles");
    particles.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::VERTEX;
    let particles = app.assets_mut().add(particles);

    // Create a buffer store our indirect draw params
    let mut indirect_params = ShaderStorageBuffer::from(DrawIndirectArgs {
        index_count: 21,
        instance_count: NUM_PARTICLES,
        first_index: 0,
        base_vertex: 0,
        first_instance: 0,
    });
    indirect_params.buffer_description.label = Some("indirect_params");
    indirect_params.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::INDIRECT;
    let indirect_params = app.assets_mut().add(indirect_params);

    Model {
        particles,
        indirect_params,
        circle_radius: 0.5,
    }
}

fn update(app: &App, model: &mut Model) {
    if app.keys().pressed(KeyCode::ArrowUp) {
        model.circle_radius += 0.01;
    }
    if app.keys().pressed(KeyCode::ArrowDown) {
        model.circle_radius -= 0.01;
    }
}

fn compute(app: &App, model: &Model, state: State, view: Entity) -> (State, ComputeModel) {
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
        particle_count: NUM_PARTICLES,
        resolution: window.size_pixels(),
    };

    match state {
        State::Init => (State::Update, compute_model),
        State::Update => (State::Update, compute_model),
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(GRAY);

    let draw = draw.material(model.material());
    draw.indirect()
        .primitive(draw.ellipse().w_h(5.0, 5.0))
        .buffer(model.indirect_params.clone());
}
