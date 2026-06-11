//! A simple and expressive API for drawing 2D and 3D graphics, built on [Bevy].
//!
//! The heart of this crate is [`Draw`] - a state machine that records drawing
//! commands (shapes, paths, meshes, text and textures) and converts them into meshes for
//! rendering. Construct one via `Draw::new`, chain calls like `draw.ellipse()` or
//! `draw.background()` to describe a scene, and let the renderer turn it into Bevy meshes.
//!
//! Add the [`NannouDrawPlugin`] to a Bevy `App` to enable the `Draw` API: it attaches a [`Draw`]
//! component to each window and renders the recorded commands each frame via the
//! [`render`] module. The [**nannou**](https://docs.rs/nannou) crate re-exports this API and adds
//! this plugin for you, so most users will not need to depend on `nannou_draw` directly.
//!
//! [Bevy]: https://bevyengine.org

use crate::render::{NannouRenderPlugin, NannouShaderModel};
use bevy::prelude::*;
use draw::Draw;
use text::font::SharedTextCx;

pub mod color;
pub mod draw;
pub mod render;
pub mod text;

pub struct NannouDrawPlugin;

impl Plugin for NannouDrawPlugin {
    fn build(&self, app: &mut App) {
        // Text rendering rasterises glyphs via `bevy_text`'s atlas machinery.
        if !app.is_plugin_added::<bevy::text::TextPlugin>() {
            app.add_plugins(bevy::text::TextPlugin);
        }
        text::font::init_shared_text_cx(app);
        app.add_plugins(NannouRenderPlugin)
            .add_systems(First, (spawn_draw, reset_draw).chain());
    }
}

fn reset_draw(mut draw_q: Query<&mut Draw>) {
    for mut draw in draw_q.iter_mut() {
        draw.reset();
    }
}

fn spawn_draw(
    mut commands: Commands,
    query: Query<Entity, (Without<Draw>, With<Window>)>,
    text_cx: Res<SharedTextCx>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(Draw::<NannouShaderModel>::new(entity, text_cx.clone()));
    }
}
