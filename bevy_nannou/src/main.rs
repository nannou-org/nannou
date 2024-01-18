use bevy_nannou::NannouPlugin;

pub fn main() {
    let mut app = bevy::app::App::new();
    app.add_plugins((bevy::DefaultPlugins, NannouPlugin));
    app.run();
}