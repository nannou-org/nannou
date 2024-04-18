use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource as wgpu;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::window::WindowRef;
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};

use crate::draw::mesh::MeshExt;
use crate::draw::primitive::Primitive;
use crate::draw::render::{GlyphCache, RenderContext, RenderPrimitive};
use crate::draw::Context;
use nannou_core::math::map_range;

pub struct NannouRenderPlugin;

impl Plugin for NannouRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_default_texture)
            .add_plugins((
                ExtractComponentPlugin::<NannouTextureHandle>::default(),
                MaterialPlugin::<DefaultNannouMaterial>::default(),
            ))
            .add_plugins(ExtractResourcePlugin::<DefaultTextureHandle>::default())
            .insert_resource(GlyphCache::new([1024; 2], 0.1, 0.1))
            .add_systems(Update, texture_event_handler)
            .add_systems(
                Last,
                (
                    update_background_color,
                    // update_draw_mesh::<M>
                ),
            );
    }
}

// ----------------------------------------------------------------------------
// Components and Resources
// ----------------------------------------------------------------------------

pub type DefaultNannouMaterial = ExtendedMaterial<StandardMaterial, NannouMaterial<"", "">>;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct NannouMaterial<const VS: &'static str, const FS: &'static str> {}

impl<const VS: &'static str, const FS: &'static str> MaterialExtension for NannouMaterial<VS, FS> {
    fn vertex_shader() -> ShaderRef {
        if !VS.is_empty() {
            VS.into()
        } else {
            ShaderRef::Default
        }
    }

    fn fragment_shader() -> ShaderRef {
        if !FS.is_empty() {
            FS.into()
        } else {
            ShaderRef::Default
        }
    }
}

#[derive(Resource, Deref, DerefMut, ExtractResource, Clone)]
pub struct DefaultTextureHandle(Handle<Image>);

#[derive(Component, Deref, DerefMut, ExtractComponent, Clone)]
pub struct NannouTextureHandle(Handle<Image>);

fn texture_event_handler(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    assets: Res<Assets<Image>>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::Added { .. } | AssetEvent::Modified { .. } | AssetEvent::Removed { .. } => {
                // TODO: handle these events
            }
            AssetEvent::LoadedWithDependencies { id } => {
                let handle = Handle::Weak(*id);
                let image = assets.get(&handle).unwrap();
                // TODO hack to only handle 2D textures for now
                // We should maybe require users to spawn a NannouTextureHandle themselves
                if image.texture_descriptor.dimension == wgpu::TextureDimension::D2 {
                    commands.spawn(NannouTextureHandle(handle));
                }
            }
            _ => {}
        }
    }
}

fn setup_default_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture = images.add(Image::default());
    commands.insert_resource(DefaultTextureHandle(texture));
}

fn update_background_color(
    mut cameras_q: Query<(&mut Camera)>,
    draw_q: Query<(Entity, &crate::draw::BackgroundColor)>,
) {
    for (entity, bg_color) in draw_q.iter() {
        for (mut camera) in cameras_q.iter_mut() {
            if let RenderTarget::Window(WindowRef::Entity(window_target)) = camera.target {
                if window_target == entity {
                    camera.clear_color = ClearColorConfig::Custom(bg_color.0);
                }
            }
        }
    }
}

// Prepare our mesh for rendering
fn update_draw_mesh<M: Material>(
    mut commands: Commands,
    mut glyph_cache: ResMut<GlyphCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<M>>,
    draw_q: Query<(&Primitive, &Handle<M>, &Context)>,
    windows_q: Query<&Window>,
    mut mesh_q: Query<(&Handle<Mesh>, &mut Transform), With<NannouMesh>>,
) {
    // for window in windows_q.iter() {
    //     for (primitive, material, context) in &draw_q {
    //         // TODO: Unclear if we need to track this, or if the physical size is enough.
    //         let scale_factor = 1.0;
    //         let [w_px, h_px] = [window.physical_width(), window.physical_height()];
    //
    //         // Converting between pixels and points.
    //         let px_to_pt = |s: u32| s as f32 / scale_factor;
    //
    //         // TODO: Store these in `Renderer`.
    //         let mut fill_tessellator = FillTessellator::new();
    //         let mut stroke_tessellator = StrokeTessellator::new();
    //
    //         // Collect all draw commands to avoid borrow errors.
    //         let draw_cmds: Vec<_> = draw.drain_commands().collect();
    //
    //         let draw_state = draw.state.write().expect("failed to lock draw state");
    //         let intermediary_state = draw_state
    //             .intermediary_state
    //             .read()
    //             .expect("failed to lock intermediary state");
    //
    //         let mut prim_idx = 0;
    //         for cmd in draw_cmds.into_iter() {
    //             match cmd {
    //                 draw::DrawCommand::Context(ctxt) => curr_ctxt = ctxt,
    //                 draw::DrawCommand::Primitive(prim) => {
    //                     // Info required during rendering.
    //                     let ctxt = RenderContext {
    //                         intermediary_mesh: &intermediary_state.intermediary_mesh,
    //                         path_event_buffer: &intermediary_state.path_event_buffer,
    //                         path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
    //                         path_points_textured_buffer: &intermediary_state
    //                             .path_points_textured_buffer,
    //                         text_buffer: &intermediary_state.text_buffer,
    //                         theme: &draw_state.theme,
    //                         transform: &curr_ctxt.transform,
    //                         fill_tessellator: &mut fill_tessellator,
    //                         stroke_tessellator: &mut stroke_tessellator,
    //                         glyph_cache: &mut glyph_cache,
    //                         output_attachment_size: Vec2::new(px_to_pt(w_px), px_to_pt(h_px)),
    //                         output_attachment_scale_factor: scale_factor,
    //                     };
    //
    //                     // Get or spawn the mesh and material.
    //                     let (mesh, material) = match mesh_q.iter_mut().nth(prim_idx) {
    //                         // We already have a mesh and material for this index.
    //                         Some((mesh, material, mut transform)) => {
    //                             transform.translation = Vec3::new(0.0, 0.0, prim_idx as f32 * 0.0001);
    //                             (mesh.clone(), material.clone())
    //                         }
    //                         // We need to spawn a new mesh and material for this index.
    //                         None => {
    //                             let mesh = Mesh::init_with_topology(curr_ctxt.topology);
    //                             let mesh = meshes.add(mesh);
    //                             let material = materials.add(StandardMaterial::default());
    //
    //                             commands.spawn((
    //                                 NannouMesh,
    //                                 PbrBundle {
    //                                     mesh: mesh.clone(),
    //                                     material: material.clone(),
    //                                     transform: Transform::from_translation(Vec3::new(
    //                                         0.0,
    //                                         0.0,
    //                                         prim_idx as f32 * 0.0001,
    //                                     )),
    //                                     ..default()
    //                                 },
    //                             ));
    //
    //                             (mesh, material)
    //                         }
    //                     };
    //
    //                     // Fetch the mesh and material.
    //                     let (mesh, material) = (
    //                         meshes.get_mut(&mesh).unwrap(),
    //                         materials.get_mut(&material).unwrap(),
    //                     );
    //
    //                     // Render the primitive.
    //                     mesh.clear();
    //                     let render = prim.render_primitive(ctxt, mesh);
    //                     // material.base_color_texture = render.texture_handle;
    //                     prim_idx += 1;
    //                 }
    //             }
    //         }
    //     }
    // }
}

#[derive(Component)]
pub struct NannouMesh;

#[derive(Component)]
pub struct NannouPersistantMesh;