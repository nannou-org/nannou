use std::collections::HashMap;
use std::ops::{Deref, Range};

use bevy::asset::load_internal_asset;
use bevy::core::cast_slice;
use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::core_3d::{Transparent3d, CORE_3D};
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::{ExtractedCamera, NormalizedRenderTarget, RenderTarget};
use bevy::render::extract_component::{
    ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin,
};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets};
use bevy::render::render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner};
use bevy::render::render_phase::{AddRenderCommand, RenderPhase};
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

use bevy_nannou_draw::draw::mesh::vertex;
use bevy_nannou_draw::draw::render::{
    GlyphCache, RenderContext, RenderPrimitive, Scissor, VertexMode,
};
use bevy_nannou_draw::{draw, Draw};
use nannou_core::geom;
use nannou_core::math::map_range;

use crate::pipeline::{
    queue_draw_mesh_items, DrawDrawMeshItem3d, NannouPipeline, NannouPipelineKey,
};

mod pipeline;

pub struct NannouRenderPlugin;

pub const NANNOU_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(43700360588854283521);

impl Plugin for NannouRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            NANNOU_SHADER_HANDLE,
            "shaders/nannou.wgsl",
            Shader::from_wgsl
        );

        app.add_systems(Startup, setup_default_texture)
            .add_plugins(UniformComponentPlugin::<DrawMeshUniform>::default())
            .add_plugins(RenderAssetPlugin::<DrawMesh>::default())
            .add_plugins((
                ExtractComponentPlugin::<Draw>::default(),
                ExtractComponentPlugin::<DrawMeshHandle>::default(),
                ExtractComponentPlugin::<NannouTextureHandle>::default(),
            ))
            .add_plugins(ExtractResourcePlugin::<DefaultTextureHandle>::default())
            .insert_resource(GlyphCache::new([1024; 2], 0.1, 0.1))
            .init_asset::<DrawMesh>()
            .add_systems(PreUpdate, clear_draw_items)
            .add_systems(Update, texture_event_handler)
            .add_systems(Last, (add_meshes, update_draw_mesh).chain());
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<NannouPipeline>>()
            .init_resource::<TextureBindGroupCache>()
            .add_render_command::<Transparent3d, DrawDrawMeshItem3d>()
            .add_systems(ExtractSchedule, extract_draw_items)
            .add_systems(
                Render,
                (
                    prepare_default_texture_bind_group,
                    prepare_texture_bind_groups,
                    prepare_draw_mesh_uniform_bind_group,
                )
                    .in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(Render, queue_draw_mesh_items.in_set(RenderSet::Queue))
            .init_resource::<NannouPipeline>();
    }
}

#[derive(Resource, Deref, DerefMut, ExtractResource, Clone)]
struct DefaultTextureHandle(Handle<Image>);

#[derive(Component, Deref, DerefMut, ExtractComponent, Clone)]
struct NannouTextureHandle(Handle<Image>);

#[derive(Component, ExtractComponent, Clone)]
pub struct DrawMeshItem {
    scissor: Option<Scissor>,
    texture: Handle<Image>,
    vertex_mode: VertexMode,
    blend: wgpu::BlendState,
    topology: wgpu::PrimitiveTopology,
    index_range: Range<u32>,
}

#[derive(Component, ExtractComponent, Clone)]
pub struct DrawMeshHandle(Handle<DrawMesh>);

/// Create a mesh asset for every draw instance
fn add_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<DrawMesh>>,
    draw: Query<(Entity, Option<&DrawMeshHandle>), With<Draw>>,
) {
    for (entity, handle) in draw.iter() {
        if let None = handle {
            let mesh = DrawMesh::default();
            let mesh = meshes.add(mesh);
            commands.entity(entity).insert(DrawMeshHandle(mesh));
        }
    }
}

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

fn prepare_default_texture_bind_group(
    images: Res<RenderAssets<Image>>,
    default_texture_handle: Res<DefaultTextureHandle>,
    render_device: Res<RenderDevice>,
    nannou_pipeline: Res<NannouPipeline>,
    mut texture_bind_group_cache: ResMut<TextureBindGroupCache>,
) {
    let texture = images.get(&**default_texture_handle).unwrap();
    let bind_group = NannouPipeline::create_texture_bind_group(
        &render_device,
        &nannou_pipeline.texture_bind_group_layout,
        &texture.sampler,
        &texture.texture_view,
    );
    texture_bind_group_cache.insert((**default_texture_handle).clone(), bind_group);
}

fn prepare_texture_bind_groups(
    gpu_images: Res<RenderAssets<Image>>,
    mut texture_bind_group_cache: ResMut<TextureBindGroupCache>,
    render_device: Res<RenderDevice>,
    nannou_textures: Query<&NannouTextureHandle>,
) {
    // TODO: maybe don't use components for this? we don't really
    // need to run this on every frame, just when the textures change
    for texture in nannou_textures.iter() {
        if !texture_bind_group_cache.contains_key(&**texture) {
            if let Some(gpu_image) = gpu_images.get(&**texture) {
                let bind_group_layout = NannouPipeline::create_texture_bind_group_layout(
                    &render_device,
                    true,
                    wgpu::TextureSampleType::Float { filterable: true },
                );
                let bind_group = NannouPipeline::create_texture_bind_group(
                    &render_device,
                    &bind_group_layout,
                    &gpu_image.sampler,
                    &gpu_image.texture_view,
                );
                texture_bind_group_cache.insert((**texture).clone(), bind_group);
            }
        }
    }
}

fn extract_draw_items(mut commands: Commands, items: Extract<Query<(Entity, &DrawMeshItem)>>) {
    for (entity, item) in items.iter() {
        commands.get_or_spawn(entity).insert((
            item.clone(),
            DrawMeshUniform {
                vertex_mode: item.vertex_mode as u32,
            },
        ));
    }
}

fn clear_draw_items(mut commands: Commands, items: Query<Entity, With<DrawMeshItem>>) {
    for entity in items.iter() {
        commands.entity(entity).despawn();
    }
}

// Prepare our mesh for rendering
fn update_draw_mesh(
    mut commands: Commands,
    mut glyph_cache: ResMut<GlyphCache>,
    windows: Query<(&Window, Has<PrimaryWindow>)>,
    msaa: Res<Msaa>,
    default_texture_handle: Res<DefaultTextureHandle>,
    mut meshes: ResMut<Assets<DrawMesh>>,
    draw: Query<(&Draw, &DrawMeshHandle, &Camera)>,
) {
    for (draw, handle, camera) in &draw {
        let Some(mut mesh) = meshes.get_mut(&handle.0) else {
            continue;
        };
        mesh.clear();
        let mut items: Vec<DrawMeshItem> = Vec::new();

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
                    primary_window.unwrap()
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
        let mut curr_start_index = 0;
        // Track whether new commands are required.
        let mut curr_scissor = None;
        let mut curr_texture_handle: Handle<Image> = (**default_texture_handle).clone();
        let mut curr_mode = VertexMode::Color;

        // Collect all draw commands to avoid borrow errors.
        let draw_cmds: Vec<_> = draw.drain_commands().collect();
        let draw_state = draw.state.write().expect("failed to lock draw state");
        let intermediary_state = draw_state
            .intermediary_state
            .read()
            .expect("failed to lock intermediary state");
        for cmd in draw_cmds {
            match cmd {
                draw::DrawCommand::Context(ctxt) => curr_ctxt = ctxt,
                draw::DrawCommand::Primitive(prim) => {
                    // Track the prev index and vertex counts.
                    let prev_index_count = mesh.indices().len() as u32;
                    let prev_vert_count = mesh.vertex_count();

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

                    // Render the primitive.
                    let render = prim.render_primitive(ctxt, &mut mesh);

                    // If the mesh indices are unchanged, there's nothing to be drawn.
                    if prev_index_count == mesh.indices().len() as u32 {
                        assert_eq!(
                            prev_vert_count,
                            mesh.vertex_count(),
                            "vertices were submitted during `render` without submitting indices",
                        );
                        continue;
                    }

                    curr_texture_handle = match render.texture_handle {
                        Some(handle) => handle,
                        None => {
                            // If there is no texture, use the default texture.
                            (**default_texture_handle).clone()
                        }
                    };

                    let new_pipeline_key = {
                        let topology = curr_ctxt.topology;
                        NannouPipelineKey {
                            output_color_format: if camera.hdr {
                                ViewTarget::TEXTURE_FORMAT_HDR
                            } else {
                                wgpu::TextureFormat::bevy_default()
                            },
                            sample_count: msaa.samples(),
                            topology,
                            blend_state: curr_ctxt.blend,
                        }
                    };

                    let new_scissor = curr_ctxt.scissor;
                    let scissor_changed = Some(new_scissor) != curr_scissor;

                    // If necessary, push a new scissor command.
                    let scissor = if scissor_changed {
                        curr_scissor = Some(new_scissor);
                        let rect = match curr_ctxt.scissor {
                            draw::Scissor::Full => full_rect,
                            draw::Scissor::Rect(rect) => full_rect
                                .overlap(rect)
                                .unwrap_or(geom::Rect::from_w_h(0.0, 0.0)),
                            draw::Scissor::NoOverlap => geom::Rect::from_w_h(0.0, 0.0),
                        };
                        let [left, bottom] = window_to_scissor(rect.bottom_left().into());
                        let (width, height) = rect.w_h();
                        let (width, height) = (pt_to_px(width), pt_to_px(height));
                        let scissor = Scissor {
                            left,
                            bottom,
                            width,
                            height,
                        };
                        Some(scissor)
                    } else {
                        None
                    };

                    // Extend the vertex mode channel.
                    curr_mode = render.vertex_mode;
                    items.push(DrawMeshItem {
                        scissor,
                        texture: curr_texture_handle.clone(),
                        vertex_mode: curr_mode,
                        blend: curr_ctxt.blend,
                        topology: curr_ctxt.topology,
                        index_range: curr_start_index..mesh.indices().len() as u32,
                    });
                    curr_start_index = mesh.indices().len() as u32;
                }
            }
        }

        // Insert a final item if there is still draw data.
        items.push(DrawMeshItem {
            scissor: None,
            texture: curr_texture_handle.clone(),
            vertex_mode: curr_mode,
            blend: curr_ctxt.blend,
            topology: curr_ctxt.topology,
            index_range: curr_start_index..mesh.indices().len() as u32,
        });
        commands.spawn_batch(items);
    }
}

#[derive(Component, ShaderType, Clone, Copy)]
pub struct DrawMeshUniform {
    vertex_mode: u32,
}

// Resource wrapper for our view uniform bind group
#[derive(Resource)]
pub struct DrawMeshUniformBindGroup {
    bind_group: wgpu::BindGroup,
}

impl DrawMeshUniformBindGroup {
    fn new(
        device: &RenderDevice,
        layout: &wgpu::BindGroupLayout,
        binding: wgpu::BindingResource,
    ) -> DrawMeshUniformBindGroup {
        let bind_group = bevy_nannou_wgpu::BindGroupBuilder::new()
            .binding(binding)
            .build(device, layout);

        DrawMeshUniformBindGroup { bind_group }
    }
}

fn prepare_draw_mesh_uniform_bind_group(
    mut commands: Commands,
    pipeline: Res<NannouPipeline>,
    render_device: Res<RenderDevice>,
    uniforms: Res<ComponentUniforms<DrawMeshUniform>>,
) {
    if let Some(binding) = uniforms.uniforms().binding() {
        commands.insert_resource(DrawMeshUniformBindGroup::new(
            &render_device,
            &pipeline.mesh_bind_group_layout,
            binding,
        ));
    }
}

#[derive(Asset, Deref, DerefMut, TypePath, Clone, Default, Debug)]
pub struct DrawMesh(draw::Mesh);

#[derive(Debug, Clone)]
pub struct GpuDrawMesh {
    index_buffer: wgpu::Buffer,
    point_buffer: wgpu::Buffer,
    color_buffer: wgpu::Buffer,
    tex_coords_buffer: wgpu::Buffer,
}

impl RenderAsset for DrawMesh {
    type ExtractedAsset = DrawMesh;
    type PreparedAsset = GpuDrawMesh;
    type Param = SRes<RenderDevice>;

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        mesh: Self::ExtractedAsset,
        render_device: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let colors = mesh
            .colors()
            .iter()
            .map(|c| Vec4::new(c.red, c.green, c.blue, c.alpha))
            .collect::<Vec<Vec4>>();
        let vertex_usage = wgpu::BufferUsages::VERTEX;
        let points_bytes = cast_slice(&mesh.points()[..]);
        let colors_bytes = cast_slice(&colors);
        let tex_coords_bytes = cast_slice(&mesh.tex_coords());
        let indices_bytes = cast_slice(&mesh.indices());
        let point_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer point_buffer"),
            contents: points_bytes,
            usage: vertex_usage,
        });
        let color_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer color_buffer"),
            contents: colors_bytes,
            usage: vertex_usage,
        });
        let tex_coords_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer tex_coords_buffer"),
            contents: tex_coords_bytes,
            usage: vertex_usage,
        });
        let index_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("nannou Renderer index_buffer"),
            contents: indices_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(GpuDrawMesh {
            index_buffer,
            point_buffer,
            color_buffer,
            tex_coords_buffer,
        })
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct TextureBindGroupCache(HashMap<Handle<Image>, wgpu::BindGroup>);
