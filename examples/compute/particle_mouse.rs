use nannou::prelude::bevy_render::renderer::RenderDevice;
use nannou::prelude::bevy_render::storage::ShaderStorageBuffer;
use nannou::prelude::*;
use std::sync::Arc;

const NUM_PARTICLES: u32 = 1000;
const WORKGROUP_SIZE: u32 = 64;

fn main() {
    nannou::app(model).compute(compute).update(update).run();
}

struct Model {
    particles: Handle<ShaderStorageBuffer>,
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: Vec4,
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
    mouse: Vec2,
    #[uniform(2)]
    resolution: Vec2,
}

impl Compute for ComputeModel {
    type State = State;

    fn shader() -> ShaderRef {
        "shaders/particle_mouse.wgsl".into()
    }

    fn entry(state: &Self::State) -> &'static str {
        match state {
            State::Init => "init",
            State::Update => "update",
        }
    }

    fn workgroup_size(_state: &Self::State) -> (u32, u32, u32) {
        (WORKGROUP_SIZE, 1, 1)
    }
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Default)]
struct DrawMaterial {
    #[storage(0, visibility(vertex))]
    particles: Handle<ShaderStorageBuffer>,
}

impl Material for DrawMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/particle_mouse.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/particle_mouse.wgsl".into()
    }
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
        NUM_PARTICLES as usize * particle_size,
        RenderAssetUsages::RENDER_WORLD,
    );
    particles.buffer_description.usage |= BufferUsages::STORAGE | BufferUsages::VERTEX;

    let particles = app.assets_mut().add(particles.clone());

    Model { particles }
}

fn update(app: &App, model: &mut Model) {

}

fn compute(app: &App, model: &Model, state: State, view: Entity) -> (State, ComputeModel) {
    let window = app.main_window();
    let window_rect = window.rect();

    let mouse_pos = app.mouse();
    let compute_model = ComputeModel {
        particles: model.particles.clone(),
        mouse: mouse_pos,
        resolution: Vec2::new(window_rect.w(), window_rect.h()),
    };

    match state {
        State::Init => (State::Update, compute_model),
        State::Update => (State::Update, compute_model),
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw().material(DrawMaterial {
        particles: model.particles.clone(),
    });
    draw.background().color(BLACK);

    for _ in 0..NUM_PARTICLES {
        draw.rect()
            .w_h(1.0, 1.0);
    }
}
