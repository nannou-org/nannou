//! Demonstrates nannou's `App` system param.
//!
//! `nannou::context::App` bundles nannou's conveniences (time, input, the focused window and the
//! `Draw` API) into an ordinary Bevy [`SystemParam`]. This lets you reach for the familiar nannou
//! helpers from your own Bevy systems - without any of the classic `nannou::app`/`sketch` builder
//! machinery, and without any `unsafe`.
//!
//! Run with: `cargo run --example system_param`
use bevy::prelude::App;
use bevy::prelude::*;
use nannou::NannouPlugin;
use nannou::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, NannouPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(app: nannou::context::App) {
    // Spawn a camera so the focused window gets a `Draw` we can render to. `App::new_camera`
    // builds a `NannouCamera` for us; `commands.spawn(render::NannouCamera)` would work too.
    app.new_camera().build();
}

// `nannou::context::App` is a normal Bevy system param, so it composes with any others you need.
fn update(app: nannou::context::App) {
    let draw = app.draw();
    draw.background().color(DIM_GRAY);

    // An ellipse that follows the mouse and pulses with elapsed time.
    let radius = 50.0 + 25.0 * app.time().sin();
    draw.ellipse()
        .xy(app.mouse())
        .radius(radius)
        .color(CORNFLOWER_BLUE);
}
