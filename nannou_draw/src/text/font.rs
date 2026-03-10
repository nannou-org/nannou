//! Font loading and shared text context.

use std::sync::{Arc, Mutex};

use bevy::asset::{AssetEvent, Assets};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use parley::FontContext;
use parley::LayoutContext;

/// Font database and layout scratch space.
pub struct NannouTextCxInner {
    pub font: FontContext,
    pub layout: LayoutContext<Color>,
}

/// Shared text context resource for use by `Draw` and `App`.
#[derive(Resource, Clone)]
pub struct SharedTextCx(pub Arc<Mutex<NannouTextCxInner>>);

impl Default for SharedTextCx {
    fn default() -> Self {
        let font = FontContext::new();

        // Register embedded notosans.
        #[cfg(feature = "notosans")]
        {
            font.collection
                .register_fonts(notosans::REGULAR_TTF.into());
        }

        SharedTextCx(Arc::new(Mutex::new(NannouTextCxInner {
            font,
            layout: LayoutContext::new(),
        })))
    }
}

/// Sync bevy font assets into our font collection.
pub fn sync_bevy_fonts_to_nannou(
    text_cx: Res<SharedTextCx>,
    mut events: MessageReader<AssetEvent<bevy::text::Font>>,
    fonts: Res<Assets<bevy::text::Font>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } = event {
            if let Some(font) = fonts.get(*id) {
                let mut inner = text_cx.0.lock().unwrap();
                inner
                    .font
                    .collection
                    .register_fonts(font.data.clone(), None);
            }
        }
    }
}
