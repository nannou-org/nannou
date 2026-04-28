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
        .add_systems(Update, (attach_image, mutate_image, draw_rect).chain())
        .run();
}

#[derive(Component)]
struct Holder {
    image: Handle<Image>,
}

#[derive(Component)]
struct PendingImage;

fn setup(mut commands: Commands) {
    commands.spawn(render::NannouCamera);
    commands.spawn(PendingImage);
}

fn attach_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    pending: Query<Entity, (With<PendingImage>, Without<Holder>)>,
) {
    for entity in pending.iter() {
        let image = Image::new_fill(
            Extent3d {
                width: 640,
                height: 360,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let h = images.add(image);
        commands
            .entity(entity)
            .insert(Holder { image: h })
            .remove::<PendingImage>();
    }
}

fn mutate_image(
    holders: Query<&Holder>,
    mut images: ResMut<Assets<Image>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    for holder in &holders {
        if let Some(image) = images.get_mut(&holder.image) {
            let image = image.into_inner();
            let mut pixels = Vec::with_capacity(640 * 360 * 4);
            for y in 0..360u32 {
                for x in 0..640u32 {
                    let r = (((x as f32 / 64.0).sin() * 0.5 + 0.5) * 255.0) as u8;
                    let g = (((y as f32 / 64.0).cos() * 0.5 + 0.5) * 255.0) as u8;
                    pixels.extend_from_slice(&[r, g, 64, 255]);
                }
            }
            image.data = Some(pixels);
            *done = true;
        }
    }
}

fn draw_rect(draw: Single<&Draw>, holders: Query<&Holder>) {
    draw.background().color(DIM_GRAY);
    for holder in &holders {
        draw.rect().w_h(640.0, 400.0).texture(&holder.image);
    }
}
