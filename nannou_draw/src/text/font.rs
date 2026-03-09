//! Font loading and management.
//!
//! Fonts are loaded via bevy's asset system and registered into the shared
//! `NannouTextCx` resource which wraps parley's `FontContext` and `LayoutContext`.

use bevy::asset::io::Reader;
use bevy::asset::{AssetEvent, AssetLoader, Assets, LoadContext};
use bevy::prelude::*;
use parley::FontContext;
use parley::LayoutContext;

/// A combined text context resource containing both the font database and layout scratch space.
///
/// This is a single resource to avoid double-borrow panics when accessing via `App::resource_mut`.
#[derive(Resource)]
pub struct NannouTextCx {
    pub font: FontContext,
    pub layout: LayoutContext<Color>,
}

impl Default for NannouTextCx {
    fn default() -> Self {
        let mut font = FontContext::new();

        // Register embedded notosans if the feature is enabled.
        #[cfg(feature = "notosans")]
        {
            font.collection.register_fonts(notosans::REGULAR_TTF.into());
        }

        Self {
            font,
            layout: LayoutContext::new(),
        }
    }
}

/// A font asset that can be loaded via bevy's asset system.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct Font {
    pub data: Vec<u8>,
    pub family_name: Option<String>,
}

/// Errors that can occur when loading a font.
#[derive(Debug, thiserror::Error)]
pub enum FontLoaderError {
    #[error("failed to read font file: {0}")]
    Io(#[from] std::io::Error),
}

/// Asset loader for `.ttf` and `.otf` font files.
#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
    type Asset = Font;
    type Settings = ();
    type Error = FontLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).await?;
        Ok(Font {
            data,
            family_name: None,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}

/// System that watches for loaded font assets and registers them into the font collection.
pub fn load_font_assets_into_collection(
    mut text_cx: ResMut<NannouTextCx>,
    mut events: EventReader<AssetEvent<Font>>,
    fonts: Res<Assets<Font>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } = event {
            if let Some(font) = fonts.get(*id) {
                text_cx.font.collection.register_fonts(font.data.clone());
            }
        }
    }
}
