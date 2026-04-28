use bevy::asset::RenderAssetUsages;
use bevy::prelude::App;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use nannou::NannouPlugin;
use nannou::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, NannouPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (mutate_canary, update).chain())
        .run();
}

#[derive(Resource)]
struct Canary(Handle<Image>);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(render::NannouCamera);
    // BASELINE: no VideoPlayer at all, just the canary.

    let canary = Image::new_fill(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[200, 0, 200, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    commands.insert_resource(Canary(images.add(canary)));
}

fn mutate_canary(canary: Res<Canary>, mut images: ResMut<Assets<Image>>, time: Res<Time>) {
    if let Some(image) = images.get_mut(&canary.0) {
        let image = image.into_inner();
        let mut pixels = Vec::with_capacity(64 * 64 * 4);
        let phase = time.elapsed_secs();
        for y in 0..64u32 {
            for x in 0..64u32 {
                let r = (((x as f32 / 16.0 + phase).sin() * 0.5 + 0.5) * 255.0) as u8;
                let g = (((y as f32 / 16.0 + phase).cos() * 0.5 + 0.5) * 255.0) as u8;
                pixels.extend_from_slice(&[r, g, 64, 255]);
            }
        }
        image.data = Some(pixels);
    }
}

fn update(draw: Single<&Draw>, canary: Res<Canary>) {
    draw.background().color(DIM_GRAY);
    draw.rect().w_h(300.0, 300.0).texture(&canary.0);
}
