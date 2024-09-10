use std::any::TypeId;
use std::hash::Hash;

use bevy::asset::Asset;
use bevy::asset::UntypedAssetId;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::pbr::{
    DefaultOpaqueRendererMethod, ExtendedMaterial, MaterialExtension, MaterialExtensionKey,
    MaterialExtensionPipeline, MaterialPipeline, MaterialProperties, MeshPipelineKey,
    OpaqueRendererMethod, PreparedMaterial, StandardMaterial,
};
use bevy::prelude::TypePath;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_instances::ExtractInstancesPlugin;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin};
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroup, BlendState, OwnedBindingResource, PolygonMode,
    RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::{NoFrustumCulling, RenderLayers};
use bevy::render::RenderSet::Render;
use bevy::render::{render_resource as wgpu, RenderApp};
use bevy::window::WindowRef;
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};

use crate::draw::mesh::MeshExt;
use crate::draw::render::{RenderContext, RenderPrimitive};
use crate::draw::{DrawCommand, DrawContext};
use crate::draw::indirect::{IndirectMaterialPlugin, IndirectMesh};
use crate::DrawHolder;

pub trait ShaderModel:
    Material + AsBindGroup + Clone + Default + Sized + Send + Sync + 'static
{
    /// Returns this material's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this material's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    #[allow(unused_variables)]
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
}

pub struct NannouRenderPlugin;

impl Plugin for NannouRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_default_texture)
            .add_plugins((
                ExtractComponentPlugin::<NannouTextureHandle>::default(),
            ))
            .add_plugins(ExtractResourcePlugin::<DefaultTextureHandle>::default())
            .add_systems(Update, texture_event_handler)
            .add_systems(PostUpdate, update_draw_mesh);
    }
}

#[derive(Default)]
pub struct NannouMaterialPlugin<M: Material>(std::marker::PhantomData<M>);

impl<M> Plugin for NannouMaterialPlugin<M>
where
    M: Material + Default,
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<M>::default(),
        ))
        .add_systems(PostUpdate, update_material::<M>.after(update_draw_mesh));
    }
}

#[derive(Default)]
pub struct NannouShaderModelPlugin<SM: ShaderModel>(std::marker::PhantomData<SM>);

impl<SM> Plugin for NannouShaderModelPlugin<SM>
where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RenderAssetPlugin::<PreparedShaderModel<SM>>::default(),
            IndirectMaterialPlugin::<SM>::default(),
        ))
        .add_systems(PostUpdate, update_material::<SM>.after(update_draw_mesh));
    }
}

pub struct PreparedShaderModel<T: ShaderModel> {
    pub bindings: Vec<(u32, OwnedBindingResource)>,
    pub bind_group: BindGroup,
    pub key: T::Data,
}

impl<T: ShaderModel> RenderAsset for PreparedShaderModel<T> {
    type SourceAsset = T;

    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<T>>, T::Param);

    fn prepare_asset(
        material: Self::SourceAsset,
        (render_device, pipeline, ref mut material_param): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        match material.as_bind_group(&pipeline.material_layout, render_device, material_param) {
            Ok(prepared) => Ok(PreparedShaderModel {
                bindings: prepared.bindings,
                bind_group: prepared.bind_group,
                key: prepared.data,
            }),
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(material))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}

// ----------------------------------------------------------------------------
// Components and Resources
// ----------------------------------------------------------------------------

pub type DefaultNannouMaterial = ExtendedMaterial<StandardMaterial, NannouMaterial>;

pub type ExtendedNannouMaterial = ExtendedMaterial<StandardMaterial, NannouMaterial>;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
#[bind_group_data(NannouMaterialKey)]
pub struct NannouMaterial {
    pub polygon_mode: PolygonMode,
    pub blend: Option<BlendState>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NannouMaterialKey {
    polygon_mode: PolygonMode,
    blend: Option<BlendState>,
}

impl From<&NannouMaterial> for NannouMaterialKey {
    fn from(material: &NannouMaterial) -> Self {
        Self {
            polygon_mode: material.polygon_mode,
            blend: material.blend,
        }
    }
}

impl MaterialExtension for NannouMaterial {
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
    materials_q: Query<(Entity, &UntypedMaterialId)>,
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
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for draw in draw_q.iter() {
        let Some((mut window_camera, window_layers)) = cameras_q.iter_mut().find(|(camera, _)| {
            if let RenderTarget::Window(WindowRef::Entity(window)) = camera.target {
                if window == draw.window {
                    return true;
                }
            }

            false
        }) else {
            warn!("No camera found for window {:?}", draw.window);
            continue;
        };

        // Reset the clear color each frame.
        window_camera.clear_color = ClearColorConfig::None;

        // The window we are rendering to.
        let window = windows.get(draw.window).unwrap();

        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();

        let mut last_mat = None;
        let mut mesh = meshes.add(Mesh::init());
        let mut curr_ctx: DrawContext = Default::default();

        let draw_cmds = draw.drain_commands();
        let draw_state = draw.state.read().unwrap();
        let intermediary_state = draw_state.intermediary_state.read().unwrap();

        for cmd in draw_cmds {
            match cmd {
                DrawCommand::Primitive(prim) => {
                    // Info required during rendering.
                    let ctxt = RenderContext {
                        intermediary_mesh: &intermediary_state.intermediary_mesh,
                        path_event_buffer: &intermediary_state.path_event_buffer,
                        path_points_vertex_buffer: &intermediary_state.path_points_vertex_buffer,
                        text_buffer: &intermediary_state.text_buffer,
                        theme: &draw_state.theme,
                        transform: &curr_ctx.transform,
                        fill_tessellator: &mut fill_tessellator,
                        stroke_tessellator: &mut stroke_tessellator,
                        output_attachment_size: Vec2::new(window.width(), window.height()),
                        output_attachment_scale_factor: window.scale_factor(),
                    };

                    // Render the primitive.
                    let mut mesh = meshes.get_mut(&mesh).unwrap();
                    prim.render_primitive(ctxt, &mut mesh);
                }
                DrawCommand::Instanced(prim, range) => {
                    let ctxt = RenderContext {
                        intermediary_mesh: &intermediary_state.intermediary_mesh,
                        path_event_buffer: &intermediary_state.path_event_buffer,
                        path_points_vertex_buffer: &intermediary_state.path_points_vertex_buffer,
                        text_buffer: &intermediary_state.text_buffer,
                        theme: &draw_state.theme,
                        transform: &curr_ctx.transform,
                        fill_tessellator: &mut fill_tessellator,
                        stroke_tessellator: &mut stroke_tessellator,
                        output_attachment_size: Vec2::new(window.width(), window.height()),
                        output_attachment_scale_factor: window.scale_factor(),
                    };

                    // Render the primitive.
                    let mut mesh = Mesh::init();
                    prim.render_primitive(ctxt, &mut mesh);
                    let mesh = meshes.add(mesh);
                    let mat_id = last_mat.expect("No material set for instanced draw command");
                    // TODO: off by one???
                    for _ in range.start..range.end - 1 {
                        commands.spawn((
                            UntypedMaterialId(mat_id),
                            mesh.clone(),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            NannouMesh,
                            NoFrustumCulling,
                            window_layers.clone(),
                        ));
                    }
                }
                DrawCommand::Indirect(prim, indirect_buffer) => {
                    // Info required during rendering.
                    let ctxt = RenderContext {
                        intermediary_mesh: &intermediary_state.intermediary_mesh,
                        path_event_buffer: &intermediary_state.path_event_buffer,
                        path_points_vertex_buffer: &intermediary_state.path_points_vertex_buffer,
                        text_buffer: &intermediary_state.text_buffer,
                        theme: &draw_state.theme,
                        transform: &curr_ctx.transform,
                        fill_tessellator: &mut fill_tessellator,
                        stroke_tessellator: &mut stroke_tessellator,
                        output_attachment_size: Vec2::new(window.width(), window.height()),
                        output_attachment_scale_factor: window.scale_factor(),
                    };

                    // Render the primitive.
                    let mut mesh = Mesh::init();
                    prim.render_primitive(ctxt, &mut mesh);
                    let mesh = meshes.add(mesh);
                    let mat_id = last_mat.expect("No material set for instanced draw command");
                    commands.spawn((
                        IndirectMesh,
                        UntypedMaterialId(mat_id),
                        mesh.clone(),
                        NannouMesh,
                        window_layers.clone(),
                    ));
                }
                DrawCommand::Context(ctx) => {
                    curr_ctx = ctx;
                }
                DrawCommand::Material(mat_id) => {
                    // We switched materials, so start rendering into a new mesh
                    last_mat = Some(mat_id.clone());
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
                        NoFrustumCulling,
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
