use bevy::prelude::*;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy::color::palettes::css::*;
}

pub struct NannouPlugin;

pub use bevy_nannou_draw::Draw;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_nannou_render::NannouRenderPlugin,
            bevy_nannou_draw::NannouDrawPlugin,
        ));
    }
}
