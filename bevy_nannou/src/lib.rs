use bevy::prelude::*;

pub mod prelude {
    pub use bevy::color::palettes::css::*;
    pub use bevy::color::*;
    pub use bevy::input::mouse::MouseWheel;
    pub use bevy::prelude::*;
    pub use bevy::render::render_asset::*;
    pub use bevy::render::render_resource::*;
    pub use bevy::winit::UpdateMode;
    pub use bevy::core_pipeline::bloom::*;

    pub use bevy_nannou_draw::color::*;
    pub use bevy_nannou_draw::draw::*;
    pub use bevy_nannou_draw::render::blend::*;
    pub use bevy_nannou_draw::render::NannouMaterialPlugin;
    pub use bevy_nannou_draw::text::*;
    pub use bevy_nannou_draw::*;
}

pub struct NannouPlugin;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((bevy_nannou_draw::NannouDrawPlugin,));
    }
}
