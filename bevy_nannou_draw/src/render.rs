use crate::draw::Draw;
use crate::draw::{
    indirect::{IndirectMesh, IndirectShaderModelPlugin},
    instanced::{InstanceRange, InstancedMesh, InstancedShaderModelPlugin},
    mesh::MeshExt,
    render::{RenderContext, RenderPrimitive},
    DrawCommand, DrawContext,
};
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::lifetimeless::Read;
use bevy::render::extract_instances::ExtractInstance;
use bevy::render::storage::ShaderStorageBuffer;
use bevy::render::sync_world::MainEntity;
use bevy::window::PrimaryWindow;
use bevy::{
    asset::{load_internal_asset, Asset, UntypedAssetId},
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryFilter,
        system::{lifetimeless::SRes, SystemParamItem},
    },
    pbr::{
        DefaultOpaqueRendererMethod, DrawMesh, MeshPipeline, MeshPipelineKey, OpaqueRendererMethod,
        RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::{TypePath, *},
    render::{
        camera::RenderTarget,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_instances::{ExtractInstancesPlugin, ExtractedInstances},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::{MeshVertexBufferLayoutRef, RenderMesh},
        render_asset::{
            prepare_assets, PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets,
        },
        render_phase::{
            AddRenderCommand, BinnedRenderPhaseType, DrawFunctionId, DrawFunctions, PhaseItem,
            PhaseItemExtraIndex, RenderCommand, RenderCommandResult, SetItemPipeline,
            TrackedRenderPass, ViewBinnedRenderPhases, ViewSortedRenderPhases,
        },
        render_resource as wgpu,
        render_resource::{
            AsBindGroup, AsBindGroupError, AsBindGroupShaderType, BindGroup, BindGroupId,
            BindGroupLayout, BlendState, OwnedBindingResource, PipelineCache, PolygonMode,
            RenderPipelineDescriptor, ShaderRef, ShaderType, SpecializedMeshPipeline,
            SpecializedMeshPipelineError, SpecializedMeshPipelines,
        },
        renderer::RenderDevice,
        texture::GpuImage,
        view,
        view::{ExtractedView, NoFrustumCulling, RenderLayers, VisibilitySystems},
        RenderApp, RenderSet,
        RenderSet::Render,
    },
    window::WindowRef,
};
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};
use std::{any::TypeId, hash::Hash, marker::PhantomData};

pub const DEFAULT_NANNOU_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(3086880141013591);

pub trait ShaderModel:
    Asset + AsBindGroup + Clone + Default + Sized + Send + Sync + 'static
{
    /// Returns this shader model's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this shader model's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    #[allow(unused_variables)]
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Specializes the render pipeline descriptor for this shader model.
    fn specialize(
        pipeline: &ShaderModelPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: ShaderModelPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

#[derive(Component, Clone)]
pub struct ShaderModelHandle<SM: ShaderModel>(pub(crate) Handle<SM>);

impl<SM> ExtractInstance for ShaderModelHandle<SM>
where
    SM: ShaderModel,
{
    type QueryData = Read<ShaderModelHandle<SM>>;
    type QueryFilter = ();

    fn extract(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

pub struct NannouRenderPlugin;

impl Plugin for NannouRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            DEFAULT_NANNOU_SHADER_HANDLE,
            "nannou.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            ExtractComponentPlugin::<NannouTransient>::default(),
            ExtractComponentPlugin::<ShaderModelMesh>::default(),
            ExtractComponentPlugin::<IndirectMesh>::default(),
            ExtractComponentPlugin::<InstancedMesh>::default(),
            ExtractComponentPlugin::<DrawIndex>::default(),
            ExtractComponentPlugin::<InstanceRange>::default(),
            ExtractComponentPlugin::<ShaderStorageBufferHandle>::default(),
            NannouShaderModelPlugin::<DefaultNannouShaderModel>::default(),
        ))
        .add_systems(First, clear_previous_frame)
        .add_systems(
            PostUpdate,
            (
                update_draw_mesh,
                view::check_visibility::<With<ShaderModelMesh>>
                    .in_set(VisibilitySystems::CheckVisibility),
            ),
        );
    }
}

#[derive(Default)]
pub struct NannouShaderModelPlugin<SM: ShaderModel>(PhantomData<SM>);

impl<SM> Plugin for NannouShaderModelPlugin<SM>
where
    SM: ShaderModel + Default,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<SM>()
            .add_plugins((
                ExtractInstancesPlugin::<ShaderModelHandle<SM>>::extract_visible(),
                RenderAssetPlugin::<PreparedShaderModel<SM>>::default(),
                IndirectShaderModelPlugin::<SM>::default(),
                InstancedShaderModelPlugin::<SM>::default(),
            ))
            .add_systems(
                PostUpdate,
                update_shader_model::<SM>.after(update_draw_mesh),
            );

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawShaderModel<SM>>()
            .init_resource::<SpecializedMeshPipelines<ShaderModelPipeline<SM>>>()
            .add_systems(
                bevy::render::Render,
                queue_shader_model::<SM, With<ShaderModelMesh>, DrawShaderModel<SM>>
                    .after(prepare_assets::<PreparedShaderModel<SM>>)
                    .in_set(RenderSet::QueueMeshes),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<ShaderModelPipeline<SM>>();
    }
}

pub struct PreparedShaderModel<SM: ShaderModel> {
    pub bindings: Vec<(u32, OwnedBindingResource)>,
    pub bind_group: BindGroup,
    pub key: SM::Data,
}

impl<SM: ShaderModel> PreparedShaderModel<SM> {
    pub fn get_bind_group_id(&self) -> Option<BindGroupId> {
        Some(self.bind_group.id())
    }
}

impl<SM: ShaderModel> RenderAsset for PreparedShaderModel<SM> {
    type SourceAsset = SM;

    type Param = (SRes<RenderDevice>, SRes<ShaderModelPipeline<SM>>, SM::Param);

    fn prepare_asset(
        shader_model: Self::SourceAsset,
        (render_device, pipeline, ref mut shader_model_param): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        match shader_model.as_bind_group(
            &pipeline.shader_model_layout,
            render_device,
            shader_model_param,
        ) {
            Ok(prepared) => Ok(PreparedShaderModel {
                bindings: prepared.bindings,
                bind_group: prepared.bind_group,
                key: prepared.data,
            }),
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(shader_model))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}

/// Sets the bind group for a given [`ShaderModel`] at the configured `I` index.
pub struct SetShaderModelBindGroup<SM: ShaderModel, const I: usize>(PhantomData<SM>);
impl<P: PhaseItem, SM: ShaderModel, const I: usize> RenderCommand<P>
    for SetShaderModelBindGroup<SM, I>
{
    type Param = (
        SRes<RenderAssets<PreparedShaderModel<SM>>>,
        SRes<ExtractedInstances<ShaderModelHandle<SM>>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (models, model_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let models = models.into_inner();
        let model_instances = model_instances.into_inner();

        let Some(handle) = model_instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(model) = models.get(&handle.0) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &model.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub type DrawShaderModel<SM: ShaderModel> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetShaderModelBindGroup<SM, 2>,
    DrawMesh,
);

// ----------------------------------------------------------------------------
// Components and Resources
// ----------------------------------------------------------------------------

pub type DefaultNannouShaderModel = NannouShaderModel;

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct NannouShaderModelFlags: u32 {
        const TEXTURE                    = 1 << 0;
        const NONE                       = 0;
        const UNINITIALIZED              = 0xFFFF;
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
#[bind_group_data(NannouBindGroupData)]
#[uniform(0, NannouShaderModelUniform)]
pub struct NannouShaderModel {
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    pub polygon_mode: PolygonMode,
    pub blend: Option<BlendState>,
}

#[derive(Clone, Default, ShaderType)]
pub struct NannouShaderModelUniform {
    pub color: Vec4,
    pub flags: u32,
}

impl AsBindGroupShaderType<NannouShaderModelUniform> for NannouShaderModel {
    fn as_bind_group_shader_type(
        &self,
        _images: &RenderAssets<GpuImage>,
    ) -> NannouShaderModelUniform {
        let mut flags = NannouShaderModelFlags::NONE;
        if self.texture.is_some() {
            flags |= NannouShaderModelFlags::TEXTURE;
        }

        NannouShaderModelUniform {
            color: LinearRgba::from(self.color).to_vec4(),
            flags: flags.bits(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct NannouBindGroupData {
    polygon_mode: PolygonMode,
    blend: Option<BlendState>,
}

impl From<&NannouShaderModel> for NannouBindGroupData {
    fn from(shader_model: &NannouShaderModel) -> Self {
        Self {
            polygon_mode: shader_model.polygon_mode,
            blend: shader_model.blend,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_shader_model<SM, QF, RC>(
    draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<ShaderModelPipeline<SM>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<ShaderModelPipeline<SM>>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    (
        render_mesh_instances,
        nannou_meshes,
        mut phases,
        mut views,
        shader_models,
        extracted_instances,
    ): (
        Res<RenderMeshInstances>,
        Query<(Entity, &MainEntity, &DrawIndex), QF>,
        ResMut<ViewSortedRenderPhases<Transparent3d>>,
        Query<(Entity, &ExtractedView, &Msaa)>,
        Res<RenderAssets<PreparedShaderModel<SM>>>,
        Res<ExtractedInstances<ShaderModelHandle<SM>>>,
    ),
) where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
    QF: QueryFilter,
    RC: 'static,
{
    let draw_function = draw_functions.read().id::<RC>();

    for (view_entity, view, msaa) in &mut views {
        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let Some(phase) = phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        for (entity, main_entity, draw_idx) in &nannou_meshes {
            let Some(handle) = extracted_instances.get(main_entity) else {
                continue;
            };
            let Some(shader_model) = shader_models.get(&handle.0) else {
                continue;
            };
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let mesh_key =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let key = ShaderModelPipelineKey {
                mesh_key,
                bind_group_data: shader_model.key.clone(),
            };
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();

            phase.add(Transparent3d {
                distance: draw_idx.0 as f32,
                pipeline,
                entity: (entity, *main_entity),
                draw_function,
                batch_range: Default::default(),
                extra_index: PhaseItemExtraIndex(0),
            });
        }
    }
}

#[derive(Resource)]
pub struct ShaderModelPipeline<SM> {
    mesh_pipeline: MeshPipeline,
    shader_model_layout: BindGroupLayout,
    vertex_shader: Option<Handle<Shader>>,
    fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<SM>,
}

impl<SM: ShaderModel> FromWorld for ShaderModelPipeline<SM> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();

        ShaderModelPipeline {
            mesh_pipeline: world.resource::<MeshPipeline>().clone(),
            shader_model_layout: SM::bind_group_layout(render_device),
            vertex_shader: match <SM as ShaderModel>::vertex_shader() {
                ShaderRef::Default => Some(DEFAULT_NANNOU_SHADER_HANDLE),
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match <SM as ShaderModel>::fragment_shader() {
                ShaderRef::Default => Some(DEFAULT_NANNOU_SHADER_HANDLE),
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

pub struct ShaderModelPipelineKey<SM: ShaderModel> {
    pub mesh_key: MeshPipelineKey,
    pub bind_group_data: SM::Data,
}

impl<SM: ShaderModel> Eq for ShaderModelPipelineKey<SM> where SM::Data: PartialEq {}

impl<SM: ShaderModel> PartialEq for ShaderModelPipelineKey<SM>
where
    SM::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.mesh_key == other.mesh_key && self.bind_group_data == other.bind_group_data
    }
}

impl<SM: ShaderModel> Clone for ShaderModelPipelineKey<SM>
where
    SM::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mesh_key: self.mesh_key,
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<SM: ShaderModel> Hash for ShaderModelPipelineKey<SM>
where
    SM::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.mesh_key.hash(state);
        self.bind_group_data.hash(state);
    }
}

impl<SM: ShaderModel> SpecializedMeshPipeline for ShaderModelPipeline<SM>
where
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = ShaderModelPipelineKey<SM>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh_key, layout)?;
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }

        descriptor
            .layout
            .insert(2, self.shader_model_layout.clone());

        let pipeline = ShaderModelPipeline {
            mesh_pipeline: self.mesh_pipeline.clone(),
            shader_model_layout: self.shader_model_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: Default::default(),
        };
        SM::specialize(&pipeline, &mut descriptor, layout, key)?;
        Ok(descriptor)
    }
}

impl ShaderModel for NannouShaderModel {
    fn specialize(
        _pipeline: &ShaderModelPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: ShaderModelPipelineKey<Self>,
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

#[derive(Component, Deref)]
pub struct UntypedShaderModelId(UntypedAssetId);

fn update_shader_model<SM>(
    draw_q: Query<&Draw>,
    mut commands: Commands,
    mut models: ResMut<Assets<SM>>,
    models_q: Query<(Entity, &UntypedShaderModelId)>,
) where
    SM: ShaderModel,
{
    for draw in draw_q.iter() {
        let state = draw.state.write().unwrap();
        state.shader_models.iter().for_each(|(id, model)| {
            if id.type_id() == TypeId::of::<SM>() {
                let model = model.downcast_ref::<SM>().unwrap();
                models.insert(id.typed(), model.clone());
            }
        });
    }

    for (entity, UntypedShaderModelId(id)) in models_q.iter() {
        if id.type_id() == TypeId::of::<SM>() {
            commands
                .entity(entity)
                .insert(ShaderModelHandle(Handle::Weak(id.typed::<SM>())));
        }
    }
}

fn update_draw_mesh(
    mut commands: Commands,
    draw_q: Query<&Draw>,
    mut cameras_q: Query<(&mut Camera, &RenderLayers), With<NannouCamera>>,
    windows: Query<(&Window, Has<PrimaryWindow>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for draw in draw_q.iter() {
        let Some((mut window_camera, window_layers)) = cameras_q.iter_mut().find(|(camera, _)| {
            if let RenderTarget::Window(WindowRef::Primary) = camera.target {
                let Ok((_, is_primary)) = windows.get(draw.window) else {
                    return false;
                };
                if is_primary {
                    return true;
                }
            }
            if let RenderTarget::Window(WindowRef::Entity(window)) = camera.target {
                if window == draw.window {
                    return true;
                }
            }

            false
        }) else {
            debug!("No camera found for window {:?}", draw.window);
            continue;
        };

        // Reset the clear color each frame.
        window_camera.clear_color = ClearColorConfig::None;

        // The window we are rendering to.
        let (window, _) = windows.get(draw.window).unwrap();
        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();

        let mut last_shader_model = None;
        let mut mesh = meshes.add(Mesh::init());
        let mut curr_ctx: DrawContext = Default::default();

        let draw_cmds = draw.drain_commands();
        let draw_state = draw.state.read().unwrap();
        let intermediary_state = draw_state.intermediary_state.read().unwrap();

        for (idx, cmd) in draw_cmds.enumerate() {
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
                    let model_id =
                        last_shader_model.expect("No shader model set for instanced draw command");
                    commands.spawn((
                        InstancedMesh,
                        InstanceRange(range),
                        UntypedShaderModelId(model_id),
                        Mesh3d(mesh.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        NannouTransient,
                        NoFrustumCulling,
                        DrawIndex(idx),
                        window_layers.clone(),
                    ));
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
                    let model_id =
                        last_shader_model.expect("No shader model set for instanced draw command");
                    commands.spawn((
                        IndirectMesh,
                        ShaderStorageBufferHandle(indirect_buffer),
                        UntypedShaderModelId(model_id),
                        Mesh3d(mesh.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        NannouTransient,
                        NoFrustumCulling,
                        DrawIndex(idx),
                        window_layers.clone(),
                    ));
                }
                DrawCommand::Context(ctx) => {
                    curr_ctx = ctx;
                }
                DrawCommand::ShaderModel(model_id) => {
                    // We switched models, so start rendering into a new mesh
                    last_shader_model = Some(model_id.clone());
                    mesh = meshes.add(Mesh::init());
                    commands.spawn((
                        UntypedShaderModelId(model_id),
                        Mesh3d(mesh.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        ShaderModelMesh,
                        NannouTransient,
                        NoFrustumCulling,
                        DrawIndex(idx),
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

#[derive(Component, ExtractComponent, Clone)]
pub struct DrawIndex(pub usize);

#[derive(Component, ExtractComponent, Clone)]
pub struct NannouTransient;

fn clear_previous_frame(
    mut commands: Commands,
    bg_color_q: Query<Entity, With<BackgroundColor>>,
    meshes_q: Query<Entity, With<NannouTransient>>,
) {
    for entity in meshes_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in bg_color_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub struct ShaderModelMesh;

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
#[require(
    Camera3d,
    Projection(|| Projection::Orthographic(OrthographicProjection::default_3d())),
    RenderLayers
)]
pub struct NannouCamera;

impl NannouCamera {
    pub fn for_window(window: Entity) -> impl Bundle {
        (
            Self,
            Camera {
                target: RenderTarget::Window(WindowRef::Entity(window)),
                ..default()
            },
        )
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub struct ShaderStorageBufferHandle(pub Handle<ShaderStorageBuffer>);
