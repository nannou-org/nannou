//! Font loading and shared text context.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use parley::{FontContext, LayoutContext};

/// Font database and layout scratch space.
pub struct NannouTextCxInner {
    pub font: FontContext,
    pub layout: LayoutContext<Color>,
}

/// Shared text context resource for use by `Draw` and `App`.
#[derive(Resource, Clone)]
pub struct SharedTextCx(pub Arc<Mutex<NannouTextCxInner>>);

/// Initialise the font database shared between the bevy and nannou text contexts.
///
/// The same fontique collection backs both bevy's [`bevy::text::FontCx`] (kept in sync with
/// `Assets<Font>` by bevy's `load_font_assets_into_font_collection` system) and nannou's
/// [`SharedTextCx`] (used by the `Draw` and `App` text APIs): the collection is made shared
/// before cloning so that fonts registered through either context are visible to both.
pub(crate) fn init_shared_text_cx(app: &mut App) {
    let mut font = FontContext::default();
    font.collection.make_shared();

    #[cfg(feature = "notosans")]
    {
        let registered = font
            .collection
            .register_fonts(notosans::REGULAR_TTF.to_vec().into(), None);
        // Map the generic sans-serif family (parley's default) to the bundled Noto
        // Sans so that default text renders identically regardless of system fonts.
        if let Some((family_id, _)) = registered.first() {
            font.collection.set_generic_families(
                parley::GenericFamily::SansSerif,
                std::iter::once(*family_id),
            );
        }
    }

    // Bevy 0.19 turned `FontCx` from a tuple struct into one with a private
    // field, so build it via `Default` and set the public `context`.
    let mut font_cx = bevy::text::FontCx::default();
    font_cx.context = font.clone();
    app.insert_resource(font_cx);
    let inner = NannouTextCxInner {
        font,
        layout: LayoutContext::new(),
    };
    app.insert_resource(SharedTextCx(Arc::new(Mutex::new(inner))));
}
