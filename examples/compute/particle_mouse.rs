use bytemuck::{Pod, Zeroable};
use nannou::prelude::bevy_render::renderer::RenderDevice;
use nannou::prelude::*;
use std::sync::Arc;

const NUM_PARTICLES: u32 = 10000;
const WORKGROUP_SIZE: u32 = 64;

fn main() {
    nannou::app(model).compute(compute).update(update).run();
}

struct Model {
    particles: Buffer,
}

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
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
    #[storage(0, buffer, visibility(compute, vertex))]
    particles: Buffer,
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

    fn shader_entry(state: &Self::State) -> &'static str {
        match state {
            State::Init => "init",
            State::Update => "update",
        }
    }

    fn workgroup_size(_state: &Self::State) -> (u32, u32, u32) {
        (WORKGROUP_SIZE, 1, 1)
    }
}

#[derive(AsBindGroup, Asset, TypePath, Clone)]
struct DrawMaterial {
    #[storage(0, buffer, visibility(compute, vertex))]
    particles: Buffer,
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
    let device = app.resource_mut::<RenderDevice>();

    let particles = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("ParticleBuffer"),
        contents: bytemuck::cast_slice(&vec![Particle::default(); NUM_PARTICLES as usize]),
        usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
    });

    Model { particles }
}

fn update(app: &App, model: &mut Model) {}

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
    let draw = app.draw()
        .material(DrawMaterial {
            particles: model.particles.clone(),
        });
    draw.background()
        .color(BLACK);
    draw.polyline()
        .points(vec![Vec2::ZERO; NUM_PARTICLES as usize]);
}
