use bevy::app::{App, Plugin};
use crate::asset::VideoAssetPlugin;

mod asset;

pub mod prelude {
    pub use crate::asset::{Video, VideoLoaderSettings};
}

pub struct NannouVideoPlugin;

impl Plugin for NannouVideoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            VideoAssetPlugin
        );
    }
}