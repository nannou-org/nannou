use std::any::TypeId;
use std::ops::{Deref, DerefMut};

use bevy::asset::UntypedAssetId;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource as wgpu;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendState, PolygonMode, RenderPipelineDescriptor, ShaderRef,
    SpecializedMeshPipelineError,
};
use bevy::render::view::{NoFrustumCulling, RenderLayers};
use bevy::window::WindowRef;
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};

use nannou_core::math::map_range;

use crate::draw::instanced::InstancingPlugin;
use crate::draw::mesh::MeshExt;
use crate::draw::primitive::Primitive;
use crate::draw::render::{GlyphCache, RenderContext, RenderPrimitive};
use crate::draw::{DrawCommand, DrawContext};
use crate::{draw, DrawHolder};

pub struct NannouRenderPlugin;

impl Plugin for NannouRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_default_texture)
            .add_plugins((
                ExtractComponentPlugin::<NannouTextureHandle>::default(),
                MaterialPlugin::<DefaultNannouMaterial>::default(),
                InstancingPlugin,
            ))
            .add_plugins(ExtractResourcePlugin::<DefaultTextureHandle>::default())
            .insert_resource(GlyphCache::new([1024; 2], 0.1, 0.1))
            .add_systems(Update, (texture_event_handler))
            .add_systems(
                PostUpdate,
                (
                    update_draw_mesh,
                    update_material::<DefaultNannouMaterial>,
                    print_all_components,
                )
                    .chain(),
            );
    }
}

fn print_all_components(world: &mut World) {
    let mut mesh_query = world.query::<(Entity, &NannouMesh)>();
    for (entity, _) in mesh_query.iter(world) {
        // info!("Found a mesh! {:#?}", world.inspect_entity(entity));
    }
}

// ----------------------------------------------------------------------------
// Components and Resources
// ----------------------------------------------------------------------------

pub type DefaultNannouMaterial = ExtendedMaterial<StandardMaterial, NannouMaterial<"", "">>;

pub type ExtendedNannouMaterial<const VS: &'static str, const FS: &'static str> =
    ExtendedMaterial<StandardMaterial, NannouMaterial<VS, FS>>;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
#[bind_group_data(NannouMaterialKey)]
pub struct NannouMaterial<const VS: &'static str, const FS: &'static str> {
    pub polygon_mode: PolygonMode,
    pub blend: Option<BlendState>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NannouMaterialKey {
    polygon_mode: PolygonMode,
    blend: Option<BlendState>,
}

impl<const VS: &'static str, const FS: &'static str> From<&NannouMaterial<VS, FS>>
    for NannouMaterialKey
{
    fn from(material: &NannouMaterial<VS, FS>) -> Self {
        Self {
            polygon_mode: material.polygon_mode,
            blend: material.blend,
        }
    }
}

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

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(blend) = key.bind_group_data.blend {
            let fragment = descriptor.fragment.as_mut().unwrap();
            fragment.targets.iter_mut().for_each(|target| {
                if let Some(target) = target {
                    target.blend = Some(blend);
                }
            });
        }

        descriptor.primitive.polygon_mode = key.bind_group_data.polygon_mode;
        Ok(())
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

#[derive(Component, Deref)]
pub struct UntypedMaterialId(UntypedAssetId);

fn update_material<M>(
    draw_q: Query<&DrawHolder>,
    mut commands: Commands,
    mut materials: ResMut<Assets<M>>,
    mut materials_q: Query<(Entity, &UntypedMaterialId)>,
) where
    M: Material,
{
    for draw in draw_q.iter() {
        let state = draw.state.write().unwrap();
        state.materials.iter().for_each(|(id, material)| {
            if id.type_id() == TypeId::of::<M>() {
                let material = material.downcast_ref::<M>().unwrap();
                materials.insert(id.typed(), material.clone());
            }
        });
    }

    for (entity, UntypedMaterialId(id)) in materials_q.iter() {
        if id.type_id() == TypeId::of::<M>() {
            commands
                .entity(entity)
                .insert(Handle::Weak(id.typed::<M>()));
        }
    }
}

fn update_draw_mesh(
    mut commands: Commands,
    draw_q: Query<&DrawHolder>,
    mut cameras_q: Query<(&mut Camera, &RenderLayers), With<NannouCamera>>,
    windows: Query<&Window>,
    mut glyph_cache: ResMut<GlyphCache>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for draw in draw_q.iter() {
        let (mut window_camera, window_layers) = cameras_q
            .iter_mut()
            .find(|(camera, _)| {
                if let RenderTarget::Window(WindowRef::Entity(window)) = camera.target {
                    if window == draw.window {
                        return true;
                    }
                }

                false
            })
            .expect("No camera found for window");

        // Reset the clear color each frame.
        window_camera.clear_color = ClearColorConfig::None;

        // The window we are rendering to.
        let window = windows.get(draw.window).unwrap();

        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();

        let mut mesh = meshes.add(Mesh::init());
        let mut curr_ctx: DrawContext = Default::default();

        let draw_cmds = draw.drain_commands();
        let mut draw_state = draw.state.read().unwrap();
        let intermediary_state = draw_state.intermediary_state.read().unwrap();

        for cmd in draw_cmds {
            match cmd {
                DrawCommand::Primitive(prim) => {
                    // Info required during rendering.
                    let ctxt = RenderContext {
                        intermediary_mesh: &intermediary_state.intermediary_mesh,
                        path_event_buffer: &intermediary_state.path_event_buffer,
                        path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
                        path_points_textured_buffer: &intermediary_state
                            .path_points_textured_buffer,
                        text_buffer: &intermediary_state.text_buffer,
                        theme: &draw_state.theme,
                        transform: &curr_ctx.transform,
                        fill_tessellator: &mut fill_tessellator,
                        stroke_tessellator: &mut stroke_tessellator,
                        glyph_cache: &mut glyph_cache,
                        output_attachment_size: Vec2::new(window.width(), window.height()),
                        output_attachment_scale_factor: window.scale_factor(),
                    };

                    // Render the primitive.
                    let mut mesh = meshes.get_mut(&mesh).unwrap();
                    let render = prim.render_primitive(ctxt, &mut mesh);
                    // TODO ignore return value and set textures on the material directly
                }
                DrawCommand::Instanced(prim, instance_data) => {
                    let ctxt = RenderContext {
                        intermediary_mesh: &intermediary_state.intermediary_mesh,
                        path_event_buffer: &intermediary_state.path_event_buffer,
                        path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
                        path_points_textured_buffer: &intermediary_state
                            .path_points_textured_buffer,
                        text_buffer: &intermediary_state.text_buffer,
                        theme: &draw_state.theme,
                        transform: &curr_ctx.transform,
                        fill_tessellator: &mut fill_tessellator,
                        stroke_tessellator: &mut stroke_tessellator,
                        glyph_cache: &mut glyph_cache,
                        output_attachment_size: Vec2::new(window.width(), window.height()),
                        output_attachment_scale_factor: window.scale_factor(),
                    };

                    // Render the primitive.
                    let mut mesh = Mesh::init();
                    let render = prim.render_primitive(ctxt, &mut mesh);
                    mesh = mesh.with_removed_attribute(Mesh::ATTRIBUTE_COLOR);
                    let mesh = meshes.add(mesh);
                    commands.spawn((
                        NannouMesh,
                        mesh,
                        SpatialBundle::INHERITED_IDENTITY,
                        instance_data,
                        NoFrustumCulling,
                        window_layers.clone(),
                    ));
                }
                DrawCommand::Context(ctx) => {
                    curr_ctx = ctx;
                }
                DrawCommand::Material(mat_id) => {
                    // We switched materials, so start rendering into a new mesh
                    mesh = meshes.add(Mesh::init());
                    commands.spawn((
                        UntypedMaterialId(mat_id),
                        mesh.clone(),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        NannouMesh,
                        window_layers.clone(),
                    ));
                }
                DrawCommand::BackgroundColor(color) => {
                    window_camera.clear_color = ClearColorConfig::Custom(color);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct NannouMesh;

#[derive(Component)]
pub struct NannouPersistentMesh;

#[derive(Resource)]
pub struct NannouRender {
    pub mesh: Handle<Mesh>,
    pub entity: Entity,
    pub draw_context: DrawContext,
}

// BLEND
pub mod blend {
    use bevy::render::render_resource as wgpu;

    pub const BLEND_NORMAL: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };

    pub const BLEND_ADD: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Src,
        dst_factor: wgpu::BlendFactor::Dst,
        operation: wgpu::BlendOperation::Add,
    };

    pub const BLEND_SUBTRACT: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Src,
        dst_factor: wgpu::BlendFactor::Dst,
        operation: wgpu::BlendOperation::Subtract,
    };

    pub const BLEND_REVERSE_SUBTRACT: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Src,
        dst_factor: wgpu::BlendFactor::Dst,
        operation: wgpu::BlendOperation::ReverseSubtract,
    };

    pub const BLEND_DARKEST: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Min,
    };

    pub const BLEND_LIGHTEST: wgpu::BlendComponent = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Max,
    };
}

#[derive(Component)]
pub struct NannouCamera;
