use bevy::prelude::*;

use crate::render::NannouRenderPlugin;

pub mod color;
pub mod draw;
pub mod render;
pub mod text;

pub struct NannouDrawPlugin;

impl Plugin for NannouDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NannouRenderPlugin)
            .add_systems(First, (spawn_draw, reset_draw).chain());
    }
}

fn reset_draw(mut draw_q: Query<&mut DrawHolder>) {
    for mut draw in draw_q.iter_mut() {
        draw.reset();
    }
}

fn spawn_draw(mut commands: Commands, query: Query<Entity, (Without<DrawHolder>, With<Window>)>) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(DrawHolder(draw::Draw::new(entity)));
    }
}

#[derive(Component, Clone, Deref, DerefMut)]
pub struct DrawHolder(pub draw::Draw);
