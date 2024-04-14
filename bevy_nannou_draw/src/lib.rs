use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;

pub mod draw;
pub mod text;

pub struct NannouDrawPlugin;

impl Plugin for NannouDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, (spawn_draw, reset_draw).chain());
    }
}

fn reset_draw(mut draw_q: Query<&mut Draw>) {
    for mut draw in draw_q.iter_mut() {
        draw.reset();
    }
}

fn spawn_draw(mut commands: Commands, query: Query<Entity, Added<Window>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Draw(draw::Draw::new()));
    }
}

#[derive(Component, Clone, Deref, DerefMut, ExtractComponent)]
pub struct Draw(pub draw::Draw);
