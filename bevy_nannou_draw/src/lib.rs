pub mod draw;
pub mod text;

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;

pub struct NannouDrawPlugin;

impl Plugin for NannouDrawPlugin {
    fn build(&self, app: &mut App) {
        app.
            add_systems(PreUpdate, spawn_draw);
    }
}

fn spawn_draw(
    mut commands: Commands,
    query: Query<(Entity, &Camera), Added<Camera>>
) {
    for (entity, _camera) in query.iter() {
        commands.entity(entity).insert(Draw(draw::Draw::new()));
    }
}

#[derive(Component, Clone, Deref, DerefMut, ExtractComponent)]
pub struct Draw(pub draw::Draw);