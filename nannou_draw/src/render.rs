use crate::draw::{
    Draw, DrawCommand, DrawContext,
    indirect::{IndirectMesh, IndirectShaderModelPlugin},
    instanced::{InstanceRange, InstancedMesh, InstancedShaderModelPlugin},
    mesh::MeshExt,
    render::{RenderContext, RenderPrimitive},
};
use bevy::{
    asset::{Asset, AssetEventSystems, UntypedAssetId, load_internal_asset, uuid_handle},
    camera::{
        Hdr, RenderTarget,
        visibility::{NoFrustumCulling, RenderLayers, VisibilitySystems, add_visibility_class},
    },
    core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    ecs::{
        query::{Has, QueryFilter, QueryItem},
        system::{
            SystemParamItem,
            lifetimeless::{Read, SRes},
        },
    },
    mesh::MeshVertexBufferLayoutRef,
    pbr::{
        DrawMesh, MATERIAL_BIND_GROUP_INDEX, MeshPipeline, MeshPipelineKey, MeshPipelineSystems,
        RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
        SetMeshViewBindingArrayBindGroup, tonemapping_pipeline_key,
    },
    prelude::{TypePath, *},
    render::{
        RenderApp, RenderStartup, RenderSystems,
        batching::NoAutomaticBatching,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_instances::{ExtractInstance, ExtractInstancesPlugin, ExtractedInstances},
        mesh::RenderMesh,
        render_asset::{
            PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets, prepare_assets,
        },
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            AsBindGroup, AsBindGroupError, AsBindGroupShaderType, BindGroup, BindGroupId,
            BindGroupLayoutDescriptor, BindingResources, BlendState, PipelineCache, PolygonMode,
            RenderPipelineDescriptor, ShaderType, SpecializedMeshPipeline,
            SpecializedMeshPipelineError, SpecializedMeshPipelines,
        },
        renderer::RenderDevice,
        storage::ShaderBuffer,
        sync_world::MainEntity,
        texture::GpuImage,
        view::ExtractedView,
    },
    shader::{ShaderDefVal, ShaderRef},
    window::{PrimaryWindow, WindowRef},
};
use lyon::lyon_tessellation::{FillTessellator, StrokeTessellator};
use std::{any::TypeId, hash::Hash, marker::PhantomData};

pub const DEFAULT_NANNOU_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("f2dbf06f-38d5-47f1-8ad4-3f188d888dd0");

/// A dyn-safe view of a [`ShaderModel`] instance, allowing the draw state to store and
/// manipulate models of any type without knowing the concrete type.
pub(crate) trait ErasedShaderModel: Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_erased(&self) -> Box<dyn ErasedShaderModel>;
    fn set_texture_erased(&mut self, texture: Handle<Image>);
}

impl<SM: ShaderModel> ErasedShaderModel for SM {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_erased(&self) -> Box<dyn ErasedShaderModel> {
        Box::new(self.clone())
    }

    fn set_texture_erased(&mut self, texture: Handle<Image>) {
        self.set_texture(texture);
    }
}

pub trait ShaderModel:
    Asset + AsBindGroup + Clone + Default + Sized + Send + Sync + 'static
{
    /// Returns this shader model's vertex shader.
    ///
    /// If [`ShaderRef::Default`] is returned, the default mesh vertex shader will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this shader model's fragment shader.
    ///
    /// If [`ShaderRef::Default`] is returned, the default mesh fragment shader will be used.
    #[allow(unused_variables)]
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Set the model's primary texture, e.g. as provided via draw API methods like
    /// `draw.mesh().points_textured(..)`.
    ///
    /// The default implementation ignores the texture; models with a texture slot (like
    /// [`NannouShaderModel`]) bind it for sampling in their fragment shader.
    fn set_texture(&mut self, _texture: Handle<Image>) {}

    /// Specializes the render pipeline descriptor for this shader model.
    fn specialize(
        _pipeline: &ShaderModelPipeline<Self>,
        _descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: ShaderModelPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

#[derive(Component, Clone)]
pub struct ShaderModelAsset<SM: ShaderModel>(pub(crate) AssetId<SM>);

impl<SM> ExtractInstance for ShaderModelAsset<SM>
where
    SM: ShaderModel,
{
    type QueryData = Read<ShaderModelAsset<SM>>;
    type QueryFilter = ();

    fn extract(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
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
            ExtractComponentPlugin::<NannouMeshCamera>::default(),
            ExtractComponentPlugin::<ShaderModelMesh>::default(),
            ExtractComponentPlugin::<IndirectMesh>::default(),
            ExtractComponentPlugin::<InstancedMesh>::default(),
            ExtractComponentPlugin::<DrawIndex>::default(),
            ExtractComponentPlugin::<InstanceRange>::default(),
            ExtractComponentPlugin::<ShaderBufferHandle>::default(),
            NannouShaderModelPlugin::<DefaultNannouShaderModel>::default(),
        ))
        .init_resource::<TextModelKeepalive>()
        // Both are skipped while `DrawFrozen` is set so the last frame's meshes are
        // neither despawned (`clear_previous_frame`) nor rebuilt (`update_draw_mesh`),
        // leaving them - and the camera clear color - in place for the render graph.
        .add_systems(First, clear_previous_frame.run_if(crate::draw_active))
        // `update_draw_mesh` (re)spawns the draw meshes each frame, so it must run
        // before Bevy computes visibility - otherwise the freshly spawned meshes
        // have `ViewVisibility == false` when the render-world extraction runs and
        // get skipped (no draw output). It must also run after bevy has registered
        // newly loaded font assets so that text laid out this frame can use them.
        .add_systems(
            PostUpdate,
            update_draw_mesh
                .run_if(crate::draw_active)
                .before(VisibilitySystems::VisibilityPropagate)
                .after(bevy::text::load_font_assets_into_font_collection),
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
                ExtractInstancesPlugin::<ShaderModelAsset<SM>>::extract_visible(),
                RenderAssetPlugin::<PreparedShaderModel<SM>>::default(),
                IndirectShaderModelPlugin::<SM>::default(),
                InstancedShaderModelPlugin::<SM>::default(),
            ))
            .add_systems(
                PostUpdate,
                update_shader_model::<SM>
                    .after(update_draw_mesh)
                    .before(AssetEventSystems),
            );

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawShaderModel<SM>>()
            .init_resource::<SpecializedMeshPipelines<ShaderModelPipeline<SM>>>()
            .add_systems(
                bevy::render::Render,
                queue_shader_model::<SM, With<ShaderModelMesh>, DrawShaderModel<SM>>
                    .after(prepare_assets::<PreparedShaderModel<SM>>)
                    .in_set(RenderSystems::QueueMeshes),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).add_systems(
            RenderStartup,
            init_shader_model_pipeline::<SM>.after(MeshPipelineSystems),
        );
    }
}

pub struct PreparedShaderModel<SM: ShaderModel> {
    pub bindings: BindingResources,
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

    type Param = (
        SRes<RenderDevice>,
        SRes<ShaderModelPipeline<SM>>,
        SRes<PipelineCache>,
        SM::Param,
    );

    fn prepare_asset(
        shader_model: Self::SourceAsset,
        _asset_id: AssetId<Self::SourceAsset>,
        (render_device, pipeline, pipeline_cache, shader_model_param): &mut SystemParamItem<
            Self::Param,
        >,
        _previous_asset: Option<&Self>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        match shader_model.as_bind_group(
            &pipeline.shader_model_layout_descriptor,
            render_device,
            pipeline_cache,
            shader_model_param,
        ) {
            Ok(prepared) => Ok(PreparedShaderModel {
                bindings: prepared.bindings,
                bind_group: prepared.bind_group,
                key: shader_model.bind_group_data(),
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
        SRes<ExtractedInstances<ShaderModelAsset<SM>>>,
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

        let Some(model_asset) = model_instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(model) = models.get(model_asset.0) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &model.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub type DrawShaderModel<SM> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    SetShaderModelBindGroup<SM, MATERIAL_BIND_GROUP_INDEX>,
    DrawMesh,
);

// ----------------------------------------------------------------------------
// Components and Resources
// ----------------------------------------------------------------------------

pub type DefaultNannouShaderModel = NannouShaderModel;

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct NannouShaderModelFlags: u32 {
        const TEXTURE       = 1 << 0;
        const NONE          = 0;
        const UNINITIALIZED = 0xFFFF;
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
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

impl Default for NannouShaderModel {
    fn default() -> Self {
        Self {
            color: Color::default(),
            texture: None,
            polygon_mode: PolygonMode::Fill,
            blend: Some(BlendState {
                color: blend::BLEND_NORMAL,
                alpha: blend::BLEND_NORMAL,
            }),
        }
    }
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
        Query<(Entity, &MainEntity, &DrawIndex, &NannouMeshCamera), QF>,
        ResMut<ViewSortedRenderPhases<Transparent3d>>,
        Query<(
            &ExtractedView,
            &Msaa,
            Option<&Tonemapping>,
            Option<&DebandDither>,
            Has<Hdr>,
        )>,
        Res<RenderAssets<PreparedShaderModel<SM>>>,
        Res<ExtractedInstances<ShaderModelAsset<SM>>>,
    ),
) where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
    QF: QueryFilter,
    RC: 'static,
{
    let draw_function = draw_functions.read().id::<RC>();

    for (view, msaa, tonemapping, dither, has_hdr) in &mut views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        // Mirror Bevy's mesh view-key construction so our pipeline is compatible
        // with the `mesh_view_bind_group` Bevy prepares for the view. Bevy 0.19
        // replaced the HDR key bit with the view's target format (which selects
        // the color attachment format), and keys tonemapping/dithering for
        // non-HDR views (these add the tonemapping LUT binding).
        let mut view_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_target_format(view.target_format);
        if !has_hdr {
            if let Some(tonemapping) = tonemapping {
                view_key |= MeshPipelineKey::TONEMAP_IN_SHADER;
                view_key |= tonemapping_pipeline_key(*tonemapping);
            }
            if let Some(DebandDither::Enabled) = dither {
                view_key |= MeshPipelineKey::DEBAND_DITHER;
            }
        }
        for (entity, main_entity, draw_idx, mesh_camera) in &nannou_meshes {
            // Only queue a mesh into the view of the window it was drawn to.
            if mesh_camera.0 != view.retained_view_entity.main_entity.id() {
                continue;
            }
            let Some(model_asset) = extracted_instances.get(main_entity) else {
                continue;
            };
            let Some(shader_model) = shader_models.get(model_asset.0) else {
                continue;
            };
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id()) else {
                continue;
            };
            let mesh_key = view_key
                | MeshPipelineKey::from_primitive_topology_and_strip_index(
                    mesh.primitive_topology(),
                    None,
                );
            let key = ShaderModelPipelineKey {
                mesh_key,
                bind_group_data: shader_model.key.clone(),
            };
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();

            // Bevy 0.19 split `SortedRenderPhase::add` into retained/transient
            // variants. This queue rebuilds the phase every frame (like Bevy's
            // sprite path), so use `add_transient` to clear items each frame.
            phase.add_transient(Transparent3d {
                sorting_info: TransparentSortingInfo3d::AlwaysOnTop,
                distance: draw_idx.0 as f32,
                pipeline,
                entity: (entity, *main_entity),
                draw_function,
                batch_range: Default::default(),
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

#[derive(Resource)]
pub struct ShaderModelPipeline<SM> {
    mesh_pipeline: MeshPipeline,
    shader_model_layout_descriptor: BindGroupLayoutDescriptor,
    vertex_shader: Option<Handle<Shader>>,
    fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<SM>,
}

fn init_shader_model_pipeline<SM: ShaderModel>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
    mesh_pipeline: Res<MeshPipeline>,
) {
    commands.insert_resource(ShaderModelPipeline::<SM> {
        mesh_pipeline: mesh_pipeline.clone(),
        shader_model_layout_descriptor: SM::bind_group_layout_descriptor(&render_device),
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
    });
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

        // The draw API makes no winding-order guarantees - e.g. 2D primitives may wind
        // either way depending on their orientation - so, like the pre-bevy nannou
        // renderer, cull nothing. Models may override this in their `specialize`.
        descriptor.primitive.cull_mode = None;

        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
            descriptor.vertex.shader_defs.push(ShaderDefVal::UInt(
                "MATERIAL_BIND_GROUP".into(),
                MATERIAL_BIND_GROUP_INDEX as u32,
            ));
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            let fragment = descriptor.fragment.as_mut().unwrap();
            fragment.shader = fragment_shader.clone();
            fragment.shader_defs.push(ShaderDefVal::UInt(
                "MATERIAL_BIND_GROUP".into(),
                MATERIAL_BIND_GROUP_INDEX as u32,
            ));
        }

        descriptor.set_layout(
            MATERIAL_BIND_GROUP_INDEX,
            self.shader_model_layout_descriptor.clone(),
        );

        let pipeline = ShaderModelPipeline {
            mesh_pipeline: self.mesh_pipeline.clone(),
            shader_model_layout_descriptor: self.shader_model_layout_descriptor.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: Default::default(),
        };
        SM::specialize(&pipeline, &mut descriptor, layout, key)?;
        Ok(descriptor)
    }
}

impl ShaderModel for NannouShaderModel {
    fn set_texture(&mut self, texture: Handle<Image>) {
        self.texture = Some(texture);
    }

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
                let model = model.as_any().downcast_ref::<SM>().unwrap();
                models.insert(id.typed(), model.clone()).unwrap();
            }
        });
    }

    for (entity, UntypedShaderModelId(id)) in models_q.iter() {
        if id.type_id() == TypeId::of::<SM>() {
            commands
                .entity(entity)
                .insert(ShaderModelAsset(id.typed::<SM>()));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_draw_mesh(
    mut commands: Commands,
    draw_q: Query<&Draw>,
    mut cameras_q: Query<(Entity, &mut Camera, &RenderTarget, &RenderLayers), With<NannouCamera>>,
    windows: Query<(&Window, Has<PrimaryWindow>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    text_cx: Res<crate::text::font::SharedTextCx>,
    mut font_atlas_set: ResMut<bevy::text::FontAtlasSet>,
    mut images: ResMut<Assets<Image>>,
    mut scale_cx: ResMut<bevy::text::ScaleCx>,
    mut text_models: ResMut<Assets<DefaultNannouShaderModel>>,
    mut text_model_keepalive: ResMut<TextModelKeepalive>,
) {
    for draw in draw_q.iter() {
        let Some((camera_entity, mut window_camera, _, window_layers)) =
            cameras_q.iter_mut().find(|(_, _, render_target, _)| {
                if let RenderTarget::Window(WindowRef::Primary) = render_target {
                    let Ok((_, is_primary)) = windows.get(draw.window) else {
                        return false;
                    };
                    if is_primary {
                        return true;
                    }
                }
                if let RenderTarget::Window(WindowRef::Entity(window)) = render_target {
                    if *window == draw.window {
                        return true;
                    }
                }

                false
            })
        else {
            bevy::log::debug!("No camera found for window {:?}", draw.window);
            continue;
        };

        // Reset the clear color each frame.
        window_camera.clear_color = ClearColorConfig::None;

        // The window we are rendering to.
        let (window, _) = windows.get(draw.window).unwrap();
        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();

        let mut last_shader_model = None;
        let mut current_mesh = None;
        let mut curr_ctx: DrawContext = Default::default();

        let draw_cmds = draw.drain_commands();
        let draw_state = draw.state.read().unwrap();
        let intermediary_state = draw_state.intermediary_state.read().unwrap();

        for (idx, cmd) in draw_cmds.enumerate() {
            match cmd {
                // Text renders as glyph-atlas-textured quads, so it cannot join the
                // current batch: each run of glyphs gets its own mesh entity bound to
                // the atlas texture it samples.
                DrawCommand::Primitive(crate::draw::primitive::Primitive::Text(prim)) => {
                    // End the current batch so that primitives drawn after this text
                    // get a fresh mesh entity with a higher `DrawIndex`.
                    current_mesh.take();

                    let batches = prim.render_atlas_quads(
                        &intermediary_state.text_buffer,
                        &draw_state.theme,
                        &curr_ctx.transform,
                        Vec2::new(window.width(), window.height()),
                        window.scale_factor(),
                        &text_cx,
                        &mut font_atlas_set,
                        &mut images,
                        &mut scale_cx,
                    );

                    // Base the text material on the active shader model when it is the
                    // default nannou model, preserving e.g. the current blend mode.
                    // Text drawn with a custom shader model type falls back to the
                    // default model.
                    let base = last_shader_model
                        .as_ref()
                        .and_then(|id| draw_state.shader_models.get(id))
                        .and_then(|model| model.as_any().downcast_ref::<DefaultNannouShaderModel>())
                        .cloned()
                        .unwrap_or_default();

                    for crate::draw::primitive::text::TextQuadBatch { texture, mesh } in batches {
                        let mut model = base.clone();
                        // Glyph colour is carried per-vertex; the model tints white so
                        // it passes through.
                        model.color = Color::WHITE;
                        model.texture = Some(texture);
                        let handle = text_models.add(model);
                        commands.spawn((
                            UntypedShaderModelId(handle.id().untyped()),
                            Mesh3d(meshes.add(mesh)),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            ShaderModelMesh,
                            NannouTransient,
                            NoFrustumCulling,
                            NoAutomaticBatching,
                            DrawIndex(idx),
                            window_layers.clone(),
                            NannouMeshCamera(camera_entity),
                        ));
                        text_model_keepalive.0.push(handle);
                    }
                }
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

                    // If no mesh is currently set, initialise a new one.
                    let mesh = current_mesh.get_or_insert_with(|| {
                        let mesh = meshes.add(Mesh::init());
                        let model_id =
                            last_shader_model.expect("No shader model set for draw command");
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
                            NoAutomaticBatching,
                            DrawIndex(idx),
                            window_layers.clone(),
                            NannouMeshCamera(camera_entity),
                        ));
                        mesh
                    });

                    // Render the primitive.
                    let mut mesh = meshes.get_mut(mesh).unwrap();
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
                        NoAutomaticBatching,
                        DrawIndex(idx),
                        window_layers.clone(),
                        NannouMeshCamera(camera_entity),
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
                        ShaderBufferHandle(indirect_buffer),
                        UntypedShaderModelId(model_id),
                        Mesh3d(mesh.clone()),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        NannouTransient,
                        NoFrustumCulling,
                        NoAutomaticBatching,
                        DrawIndex(idx),
                        window_layers.clone(),
                        NannouMeshCamera(camera_entity),
                    ));
                }
                DrawCommand::Context(ctx) => {
                    curr_ctx = ctx;
                }
                DrawCommand::ShaderModel(model_id) => {
                    // Drop the mesh, we'll initialise a new one if something is
                    // drawn with this shader model.
                    last_shader_model = Some(model_id.clone());
                    current_mesh.take();
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

/// The main-world entity of the [`NannouCamera`] a draw mesh was generated for.
///
/// [`queue_shader_model`] uses this to scope each mesh to its window's view,
/// rather than queuing every mesh into every camera's phase.
#[derive(Component, ExtractComponent, Clone, Copy)]
pub struct NannouMeshCamera(pub Entity);

#[derive(Component, ExtractComponent, Clone)]
pub struct NannouTransient;

/// Keeps the shader models created for text quad batches alive for the frame they
/// are drawn in; dropping the handles the following frame lets the assets clean up.
#[derive(Resource, Default)]
pub struct TextModelKeepalive(Vec<Handle<DefaultNannouShaderModel>>);

fn clear_previous_frame(
    mut commands: Commands,
    meshes_q: Query<Entity, With<NannouTransient>>,
    mut text_model_keepalive: ResMut<TextModelKeepalive>,
) {
    text_model_keepalive.0.clear();
    for entity in meshes_q.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component, ExtractComponent, Clone)]
#[component(on_add = add_visibility_class::<ShaderModelMesh>)]
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
    Projection::Orthographic(OrthographicProjection::default_3d()),
    RenderLayers
)]
pub struct NannouCamera;

impl NannouCamera {
    pub fn for_window(window: Entity) -> impl Bundle {
        (
            Self,
            Camera::default(),
            RenderTarget::Window(WindowRef::Entity(window)),
        )
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub struct ShaderBufferHandle(pub Handle<ShaderBuffer>);
