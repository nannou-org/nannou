mod components;
mod events;
mod util;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use components::*;
pub use events::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::{frame_input, register_device};

use bevy::prelude::*;

pub struct WebcamPlugin;

impl Plugin for WebcamPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WebcamDevice>()
            .register_type::<WebcamSupportedFormat>()
            .register_type::<Webcam>()
            .register_type::<WebcamStream>()
            .register_type::<WebcamError>()
            .register_type::<WebcamFormat>();

        #[cfg(not(target_arch = "wasm32"))]
        native::init(app);

        #[cfg(target_arch = "wasm32")]
        wasm::init(app);
    }
}
