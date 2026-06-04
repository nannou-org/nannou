use bevy::prelude::App;
use bevy::prelude::*;
use nannou::NannouPlugin;
use nannou::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, NannouPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(render::NannouCamera);
    let video: Handle<Video> = asset_server.load("video/file_example_MP4_640_3MG.mp4");
    commands.spawn(VideoPlayer::new(video));
}

fn update(draw: Single<&Draw>, outputs: Query<&VideoOutput>) {
    draw.background().color(DIM_GRAY);
    for output in &outputs {
        draw.rect()
            .w_h(output.size.x as f32, output.size.y as f32)
            .texture(&output.image);
    }
}
