use bevy::prelude::*;
use bevy_nannou::prelude::*;
use bevy_nannou::NannouPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, NannouPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    // We only have to spawn a camera to have access to `Draw`, which will be inserted
    // automatically by `NannouPlugin` on the main window.
    commands.spawn(render::NannouCamera);
}

// This is our update loop, which will be called every frame. We can access `Draw` here using
// the `Single` query because we only have a single camera/window.
fn update(draw: Single<&Draw>) {
    draw.background().color(DIM_GRAY);
    draw.ellipse().color(BLUE).w_h(100.0, 100.0);
}
