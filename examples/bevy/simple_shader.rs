use bevy::prelude::*;
use bevy_nannou::NannouPlugin;
use bevy_nannou::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        NannouPlugin,
        NannouShaderModelPlugin::<ShaderModel>::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, update)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(render::NannouCamera);
}

fn update(draw: Single<&Draw>, time: Res<Time>) {
    draw.background().color(DIM_GRAY);
    // Animate color by time
    let color = LinearRgba::new(
        time.elapsed_secs().sin() * 0.5 + 0.5,
        time.elapsed_secs().cos() * 0.5 + 0.5,
        0.0,
        1.0,
    );
    // This moves our draw instances to use our shader model for anything we draw
    let draw = draw.shader_model(ShaderModel { color });
    // This ellipse will use our shader
    draw.ellipse().w_h(100.0, 100.0);
}

#[shader_model(fragment = "shaders/simple_shader.wgsl")]
struct ShaderModel {
    #[uniform(0)]
    color: LinearRgba,
}
