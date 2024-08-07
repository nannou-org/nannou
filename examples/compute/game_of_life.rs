use nannou::prelude::*;

const DISPLAY_FACTOR: u32 = 4;
const SIZE: (u32, u32) = (1280 / DISPLAY_FACTOR, 720 / DISPLAY_FACTOR);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    nannou::app(model)
        .update(update)
        .compute(compute)
        .view(view)
        .run()
}

struct Model {
    texture_a: Handle<Image>,
    texture_b: Handle<Image>,
    display: Handle<Image>,
}

#[derive(Clone, Default)]
enum State {
    #[default]
    Init,
    Update(usize),
}

#[derive(AsBindGroup, Clone)]
struct ComputeModel {
    #[storage_texture(0, image_format = R32Float, access = ReadOnly)]
    texture_read: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = WriteOnly)]
    texture_write: Handle<Image>,
}

impl ComputeShader for ComputeModel {
    type State = State;

    fn compute_shader() -> ShaderRef {
        "shaders/game_of_life.wgsl".into()
    }

    fn shader_entry(state: &Self::State) -> &'static str {
        match state {
            State::Init => "init",
            State::Update(_) => "update",
        }
    }

    fn workgroup_size(state: &Self::State) -> (u32, u32, u32) {
        (SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1)
    }
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window::<Model>()
        .primary()
        .size(SIZE.0 * DISPLAY_FACTOR, SIZE.1 * DISPLAY_FACTOR)
        .build();

    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::R32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image0 = app.assets_mut().add(image.clone());
    let image1 = app.assets_mut().add(image);
    Model {
        texture_a: image0.clone(),
        texture_b: image1,
        display: image0.clone(),
    }
}

fn update(app: &App, model: &mut Model) {
    if model.display == model.texture_a {
        model.display = model.texture_b.clone();
    } else {
        model.display = model.texture_b.clone();
    }
}

fn compute(app: &App, model: &Model, state: &mut State, view: Entity) -> ComputeModel {
    match state {
        State::Init => {
            *state = State::Update(1);
            ComputeModel {
                texture_read: model.texture_a.clone(),
                texture_write: model.texture_b.clone(),
            }
        }
        State::Update(0) => {
            *state = State::Update(1);
            ComputeModel {
                texture_read: model.texture_b.clone(),
                texture_write: model.texture_a.clone(),
            }
        }
        State::Update(1) => {
            *state = State::Update(0);
            ComputeModel {
                texture_read: model.texture_b.clone(),
                texture_write: model.texture_a.clone(),
            }
        }
        State::Update(_) => panic!("Invalid state"),
    }
}

fn view(app: &App, model: &Model, view: Entity) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let window_rect = app.window_rect();
    draw.rect()
        .w_h(window_rect.w(), window_rect.h())
        .texture(&model.display);
}
