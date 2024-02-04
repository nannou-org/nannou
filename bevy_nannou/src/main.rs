use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::shape::Torus;
use bevy::prelude::*;
use bevy_nannou::NannouPlugin;
use bevy_nannou_draw::color::{RED, SALMON, SEAGREEN, SEASHELL, SKYBLUE};

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, NannouPlugin))
        .insert_resource(Msaa::default())
        .add_systems(Startup, startup)
        .add_systems(Update, (update_draw, update_mesh))
        .run();
}

#[derive(Resource, Deref, DerefMut)]
struct MyTexture(Handle<Image>);

fn startup(mut commands: Commands, assets: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Torus {
            radius: 100.0,
            ..default()
        })),
        transform: Transform::from_xyz(-200.0, -200.0, 0.0),
        ..default()
    });

    commands.spawn(
        (Camera3dBundle {
            camera: Camera {
                hdr: false,
                ..Default::default()
            },
            camera_3d: Camera3d {
                // TODO: we should manage this in the nannou plugin as function of backgrond color
                clear_color: ClearColorConfig::None,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -10.0).looking_at(Vec3::ZERO, Vec3::Z),
            projection: OrthographicProjection {
                scale: 1.0,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }),
    );

    let handle = assets.load("images/nannou.png");
    commands.insert_resource(MyTexture(handle));
}

fn update_mesh(mut handles: Query<(&Handle<Mesh>, &mut Transform)>) {
    for (_, mut transform) in handles.iter_mut() {
        transform.translation.x += 1.0;
        transform.translation.y += 1.0;
        transform.translation.z += 1.0;
        transform.translation.z += 1.0;
    }
}

fn update_draw(
    draw: Query<(&mut bevy_nannou_draw::Draw)>,
    texture_handle: Res<MyTexture>,
    images: Res<Assets<Image>>,
    time: Res<Time>,
) {
    let draw = draw.single();

    let texture = match images.get(&**texture_handle) {
        None => return,
        Some(texture) => texture,
    };

    // TODO: why is the texture rotated?
    // draw.texture(texture_handle.clone(), texture.clone());
    draw.ellipse().w_h(100.0, 100.0).color(SALMON);
    draw.ellipse()
        .x(100.0 + time.elapsed().as_millis() as f32 / 100.0)
        .w_h(100.0, 100.0)
        .color(SEASHELL);
    draw.ellipse().x(-100.0).w_h(100.0, 100.0).color(SKYBLUE);
}
