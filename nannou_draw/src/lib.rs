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
        app.add_plugins(NannouRenderPlugin)
            .init_resource::<SharedTextCx>()
            .add_systems(First, (spawn_draw, reset_draw).chain())
            .add_systems(PostUpdate, text::font::sync_bevy_fonts_to_nannou);
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
