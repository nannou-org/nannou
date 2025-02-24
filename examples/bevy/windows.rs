use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::WindowResolution;
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
    // Spawn a camera for our main window
    commands.spawn(render::NannouCamera);
}

#[derive(Component)]
pub struct WindowColor(Color);

fn update(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    draws: Query<(&Draw, Option<&WindowColor>)>,
    mut count: Local<usize>,
) {
    if keys.just_pressed(KeyCode::Space) {
        // Increment the count to track the number of windows
        *count += 1;
        // We need a render layer to make sure we only render the camera for this window
        let layer = RenderLayers::layer(*count);
        // Spawn a new window with a unique title, resolution, and background color
        let entity = commands
            .spawn((
                Window {
                    title: "Nannou".to_string(),
                    resolution: WindowResolution::new(400.0, 400.0),
                    ..Default::default()
                },
                layer.clone(),
                WindowColor(match *count {
                    1 => RED.into(),
                    2 => GREEN.into(),
                    3 => BLUE.into(),
                    _ => PURPLE.into(),
                }),
            ))
            .id();
        // Spawn a camera for our new window with the matching render layer
        commands.spawn((render::NannouCamera::for_window(entity), layer.clone()));
    }

    for (draw, window_color) in draws.iter() {
        if let Some(window_color) = window_color {
            draw.background().color(window_color.0);
        } else {
            draw.background().color(DIM_GRAY);
        }

        draw.ellipse().color(LIGHT_GRAY).w_h(100.0, 100.0);
    }
}
