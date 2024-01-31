use bevy::prelude::*;

pub struct NannouPlugin;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_nannou_render::NannouRenderPlugin,
            bevy_nannou_draw::NannouDrawPlugin,
        ));
    }
}
