use std::collections::HashMap;
use std::ops::{Deref, Range};

use bevy::asset::load_internal_asset;
use bevy::core::cast_slice;
use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::core_3d::{Transparent3d, CORE_3D};
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::shape::Cube;
use bevy::prelude::*;
use bevy::render::camera::{ExtractedCamera, NormalizedRenderTarget, RenderTarget};
use bevy::render::extract_component::{
    ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::mesh::Indices;
use bevy::render::primitives::Sphere;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets};
use bevy::render::render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner};
use bevy::render::render_phase::{AddRenderCommand, DrawFunctions, RenderPhase};
use bevy::render::render_resource::{
    BindGroupLayout, BufferInitDescriptor, CachedRenderPipelineId, PipelineCache, ShaderType,
    SpecializedRenderPipeline, SpecializedRenderPipelines,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::{
    ExtractedView, ExtractedWindow, ExtractedWindows, ViewDepthTexture, ViewTarget, ViewUniforms,
};
use bevy::render::{render_resource as wgpu, Extract, RenderSet};
use bevy::render::{Render, RenderApp};
use bevy::window::{PrimaryWindow, WindowRef};
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};
use wgpu_types::PrimitiveTopology;

use bevy_nannou_draw::draw::mesh::MeshExt;
use bevy_nannou_draw::draw::render::{
    GlyphCache, RenderContext, RenderPrimitive, Scissor, VertexMode,
};
use bevy_nannou_draw::{draw, Draw};
use nannou_core::geom;
use nannou_core::math::map_range;

pub struct NannouRenderPlugin;

impl Plugin for NannouRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_default_texture)
            .add_plugins((
                ExtractComponentPlugin::<Draw>::default(),
                ExtractComponentPlugin::<NannouTextureHandle>::default(),
            ))
            .add_plugins(ExtractResourcePlugin::<DefaultTextureHandle>::default())
            .insert_resource(GlyphCache::new([1024; 2], 0.1, 0.1))
            .add_systems(Update, texture_event_handler)
            .add_systems(Last, update_draw_mesh.chain());
    }
}

// ----------------------------------------------------------------------------
// Components and Resources
// ----------------------------------------------------------------------------

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

// Prepare our mesh for rendering
fn update_draw_mesh(
    mut commands: Commands,
    mut glyph_cache: ResMut<GlyphCache>,
    windows: Query<(&Window, Has<PrimaryWindow>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    draw: Query<(&Draw, &Camera)>,
    mesh_q: Query<(&Handle<Mesh>, &Handle<StandardMaterial>), With<NannouMesh>>,
) {
    for (draw, camera) in &draw {
        let window = if let RenderTarget::Window(window_ref) = camera.target {
            match window_ref {
                WindowRef::Primary => {
                    let mut primary_window = None;
                    for (window, is_primary) in windows.iter() {
                        if is_primary {
                            primary_window = Some(window);
                            break;
                        }
                    }
                    if let Some(primary_window) = primary_window {
                        primary_window
                    } else {
                        warn!("No primary window found");
                        continue;
                    }
                }
                WindowRef::Entity(entity) => windows.get(entity).unwrap().0,
            }
        } else {
            // TODO: handle other render targets
            // For now, we only support rendering to a window
            warn!("Unsupported render target");
            continue;
        };

        // TODO: Unclear if we need to track this, or if the physical size is enough.
        let scale_factor = 1.0;
        let [w_px, h_px] = [window.physical_width(), window.physical_height()];

        // Converting between pixels and points.
        let px_to_pt = |s: u32| s as f32 / scale_factor;
        let pt_to_px = |s: f32| (s * scale_factor).round() as u32;
        let full_rect = nannou_core::geom::Rect::from_w_h(px_to_pt(w_px), px_to_pt(h_px));

        let window_to_scissor = |v: nannou_core::geom::Vec2| -> [u32; 2] {
            let x = map_range(v.x, full_rect.left(), full_rect.right(), 0u32, w_px);
            let y = map_range(v.y, full_rect.bottom(), full_rect.top(), 0u32, h_px);
            [x, y]
        };

        // TODO: Store these in `Renderer`.
        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();

        // Keep track of context changes.
        let mut curr_ctxt = draw::Context::default();

        // Collect all draw commands to avoid borrow errors.
        let draw_cmds: Vec<_> = draw.drain_commands().collect();

        info!("draw_len {}", draw_cmds.len());

        let draw_state = draw.state.write().expect("failed to lock draw state");
        let intermediary_state = draw_state
            .intermediary_state
            .read()
            .expect("failed to lock intermediary state");
        for (i, cmd) in draw_cmds.into_iter().enumerate() {
            match cmd {
                draw::DrawCommand::Context(ctxt) => curr_ctxt = ctxt,
                draw::DrawCommand::Primitive(prim) => {
                    // Info required during rendering.
                    let ctxt = RenderContext {
                        intermediary_mesh: &intermediary_state.intermediary_mesh,
                        path_event_buffer: &intermediary_state.path_event_buffer,
                        path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
                        path_points_textured_buffer: &intermediary_state
                            .path_points_textured_buffer,
                        text_buffer: &intermediary_state.text_buffer,
                        theme: &draw_state.theme,
                        transform: &curr_ctxt.transform,
                        fill_tessellator: &mut fill_tessellator,
                        stroke_tessellator: &mut stroke_tessellator,
                        glyph_cache: &mut glyph_cache,
                        output_attachment_size: Vec2::new(px_to_pt(w_px), px_to_pt(h_px)),
                        output_attachment_scale_factor: scale_factor,
                    };

                    // Get or spawn the mesh and material.
                    let (mesh, material) = match mesh_q.iter().nth(i) {
                        // We already have a mesh and material for this index.
                        Some((mesh, material)) => (mesh.clone(), material.clone()),
                        // We need to spawn a new mesh and material for this index.
                        None => {
                            let mesh = Mesh::init_with_topology(curr_ctxt.topology);
                            let mesh = meshes.add(mesh);
                            let material = materials.add(StandardMaterial::default());

                            commands.spawn((
                                NannouMesh,
                                PbrBundle {
                                    mesh: mesh.clone(),
                                    material: material.clone(),
                                    ..default()
                                },
                            ));

                            (mesh, material)
                        }
                    };

                    // Fetch the mesh and material.
                    let (mesh, material) = (
                        meshes.get_mut(&mesh).unwrap(),
                        materials.get_mut(&material).unwrap(),
                    );

                    // Render the primitive.
                    let render = prim.render_primitive(ctxt, mesh);
                    material.base_color_texture = render.texture_handle;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct NannouMesh;
