use bevy::prelude::*;
use bevy::prelude::shape::Cube;

use bevy_nannou_render::mesh::ViewMesh;

pub struct NannouPlugin;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_plugins((
            bevy_nannou_render::NannouRenderPlugin,
            bevy_nannou_draw::NannouDrawPlugin,
        ));
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = ViewMesh::default();
    let image = assets.load("images/img.png");
    mesh.texture = Some(image);
    mesh.extend_from_slices(
        &[
            Vec3::new(-512.0, -512.0, 0.0),
            Vec3::new(512.0, -512.0, 0.0),
            Vec3::new(-512.0, 512.0, 0.0),
            Vec3::new(512.0, 512.0, 0.0),
        ],
        &[1, 0, 2, 1, 2, 3],
        &[Vec4::new(1.0, 0.0, 0.0, 1.0); 4],
        &[
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
        ],
    );
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, -10.0).looking_at(Vec3::ZERO, Vec3::Z),
            projection: OrthographicProjection {
                scale: 1.0,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        },
        mesh,
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cube::new(10.0))),
        material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
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
