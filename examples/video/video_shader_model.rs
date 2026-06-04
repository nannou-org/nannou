use bevy::prelude::App;
use bevy::prelude::*;
use nannou::NannouPlugin;
use nannou::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        NannouPlugin,
        NannouShaderModelPlugin::<VideoShaderModel>::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, update)
    .run();
}

#[shader_model(fragment = "video_model.wgsl")]
struct VideoShaderModel {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(render::NannouCamera);
    let video: Handle<Video> = asset_server.load("video/file_example_MP4_640_3MG.mp4");
    commands.spawn(VideoPlayer::new(video).with_mode(PlaybackMode::Loop));
}

fn update(draw: Single<&Draw>, players: Query<&VideoOutput>) {
    draw.background().color(DIM_GRAY);
    for output in &players {
        let draw = draw.shader_model(VideoShaderModel {
            texture: output.image.clone(),
        });
        draw.rect().w_h(640.0, 400.0);
    }
}
