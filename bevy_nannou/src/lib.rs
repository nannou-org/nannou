use bevy::prelude::*;

pub mod prelude {
    pub use bevy::color::palettes::css::*;
    pub use bevy::prelude::*;
    pub use bevy::winit::UpdateMode;
    pub use bevy_nannou_draw::render::blend::*;
    pub use bevy_nannou_draw::*;
    pub use bevy_nannou_draw::draw::*;
}

pub struct NannouPlugin;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((bevy_nannou_draw::NannouDrawPlugin,));
    }
}
