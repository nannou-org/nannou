use bevy::prelude::*;
use bevy_nannou_render::mesh::{Vertex, ViewMesh};

pub struct NannouPlugin;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_plugins((
            bevy_nannou_render::NannouRenderPlugin,
            bevy_nannou_draw::NannouDrawPlugin,
        ));
    }
}

fn setup(mut commands: Commands) {
    let mut mesh = ViewMesh::default();
    mesh.extend_from_slices(
        &[
            Vec3::new(-512.0, -384.0, 0.0),
            Vec3::new(-512.0, 384.0, 0.0),
            Vec3::new(512.0, 384.0, 0.0),
        ],
        &[0, 1, 2],
        &[Vec4::new(1.0, 0.0, 0.0, 1.0); 3],
        &[Vec2::new(0.0, 0.0); 3],
    );
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, -10.0).looking_at(Vec3::ZERO, Vec3::Z),
            projection: OrthographicProjection {
                ..Default::default()
            }
            .into(),
            ..Default::default()
        },
        mesh,
    ));
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    #[test]
    fn it_works() {
        let mut app = App::new();
        app.add_plugins((bevy::DefaultPlugins, super::NannouPlugin));
        app.update();
    }
}
