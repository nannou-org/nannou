mod components;
mod events;

#[cfg(not(target_arch = "wasm32"))]
mod native;

pub use components::*;
pub use events::*;

use bevy::prelude::*;

pub struct MidiPlugin;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MidiPort>()
            .register_type::<MidiPortDirection>()
            .register_type::<MidiInput>()
            .register_type::<MidiOutput>()
            .register_type::<MidiError>();

        #[cfg(not(target_arch = "wasm32"))]
        native::init(app);
    }
}
