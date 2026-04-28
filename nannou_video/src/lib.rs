use bevy::app::{App, Plugin};
use bevy::prelude::{AssetApp, IntoScheduleConfigs, Update};

mod asset;
mod components;
mod events;

#[cfg(not(target_arch = "wasm32"))]
mod player;
#[cfg(not(target_arch = "wasm32"))]
mod worker;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use asset::{NetworkPreset, Video, VideoAssetLoaderError, VideoLoaderSettings, VideoSource};
pub use components::{HwAccelPolicy, PlaybackMode, SeekTo, VideoOutput, VideoPlayer, VideoResize};
pub use events::{VideoEnded, VideoFailed, VideoLoaded, VideoLooped, VideoSeeked};

#[cfg(not(target_arch = "wasm32"))]
pub use video_rs::location::Url;

pub mod prelude {
    pub use crate::asset::{NetworkPreset, Video, VideoLoaderSettings, VideoSource};
    pub use crate::components::{
        HwAccelPolicy, PlaybackMode, SeekTo, VideoOutput, VideoPlayer, VideoResize,
    };
    pub use crate::events::{VideoEnded, VideoFailed, VideoLoaded, VideoLooped, VideoSeeked};
    #[cfg(not(target_arch = "wasm32"))]
    pub use video_rs::location::Url;
}

pub struct NannouVideoPlugin;

impl Plugin for NannouVideoPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Video>()
            .register_type::<VideoPlayer>()
            .register_type::<VideoOutput>()
            .register_type::<PlaybackMode>()
            .register_type::<SeekTo>()
            .register_type::<HwAccelPolicy>()
            .register_type::<VideoResize>();

        #[cfg(not(target_arch = "wasm32"))]
        {
            app.init_asset_loader::<asset::VideoLoader>()
                .add_systems(
                    Update,
                    (
                        player::attach_workers,
                        player::process_seeks,
                        player::sync_commands,
                        player::drain_frames,
                    )
                        .chain(),
                )
                .add_observer(player::on_player_removed);
        }

        #[cfg(target_arch = "wasm32")]
        {
            use bevy::render::RenderApp;
            wasm::install_registry(app);
            app.init_asset_loader::<wasm::VideoLoader>()
                .add_systems(
                    Update,
                    (
                        wasm::attach_players,
                        wasm::process_seeks,
                        wasm::sync_commands,
                        wasm::drain_events,
                        wasm::sync_positions,
                    )
                        .chain(),
                )
                .add_observer(wasm::on_player_removed);
            if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
                wasm::install_render_app(render_app);
            }
        }
    }
}
