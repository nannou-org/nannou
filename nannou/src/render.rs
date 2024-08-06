use crate::app::{ModelHolder, RenderFnRes};
use crate::frame::Frame;
use crate::prelude::bevy_render::extract_component::ExtractComponent;
use crate::prelude::bevy_render::extract_resource::extract_resource;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
    ComputePipelineDescriptor, PipelineCache, ShaderRef, SpecializedComputePipeline,
    SpecializedComputePipelines,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::{FallbackImage, GpuImage};
use bevy::render::view::{ExtractedView, ExtractedWindows, ViewTarget};
use bevy::render::{Render, RenderSet};
use bevy_nannou::prelude::{AsBindGroup, ShaderStages, StorageTextureAccess, TextureFormat};
use std::borrow::Cow;
use std::ops::Deref;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use wgpu::ComputePassDescriptor;
use crate::prelude::bevy_render::{Extract, MainWorld};

pub(crate) struct RenderPlugin<M>(std::marker::PhantomData<M>);

impl<M> Default for RenderPlugin<M>
where
    M: Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<M> Plugin for RenderPlugin<M>
where
    M: Send + Sync + Clone + 'static,
{
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) else {
            return;
        };

        render_app.add_systems(
            ExtractSchedule,
            (
                extract_resource::<RenderFnRes<M>>,
                extract_resource::<ModelHolder<M>>,
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<NannouRenderNode<M>>>(
                Core3d,
                NannouRenderNodeLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::MainTransparentPass,
                    NannouRenderNodeLabel,
                    Node3d::EndMainPass,
                ),
            );
    }
}

pub struct RenderApp<'w> {
    current_view: Option<Entity>,
    world: &'w World,
}

impl<'w> RenderApp<'w> {
    pub fn new(world: &'w World) -> Self {
        Self {
            current_view: None,
            world,
        }
    }

    /// Get the elapsed seconds since startup.
    pub fn time(&self) -> f32 {
        let time = self.world.resource::<Time>();
        time.elapsed_seconds()
    }

    /// Get the elapsed seconds since the last frame.
    pub fn time_delta(&self) -> f32 {
        let time = self.world.resource::<Time>();
        time.delta_seconds()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct NannouRenderNodeLabel;

pub(crate) struct NannouRenderNode<M>(std::marker::PhantomData<M>);

impl<M> FromWorld for NannouRenderNode<M>
where
    M: Send + Sync + Clone + 'static,
{
    fn from_world(_world: &mut World) -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<M> ViewNode for NannouRenderNode<M>
where
    M: Send + Sync + Clone + 'static,
{
    type ViewQuery = (Entity, &'static ViewTarget, &'static ExtractedView);

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_entity, view_target, extracted_view): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let render_fn = world.resource::<RenderFnRes<M>>();
        let Some(render_fn) = render_fn.as_ref() else {
            return Ok(());
        };

        let extracted_windows = world.resource::<ExtractedWindows>();
        let model = world.resource::<ModelHolder<M>>();
        let render_app = RenderApp::new(world);
        let frame = Frame::new(
            world,
            view_entity,
            view_target,
            extracted_windows,
            extracted_view,
            render_context,
        );

        render_fn(&render_app, &model.deref(), frame);

        Ok(())
    }
}
pub(crate) struct ComputePlugin<CM: ComputeShader>(std::marker::PhantomData<CM>);

impl<CM: ComputeShader> Default for ComputePlugin<CM> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<CM> Plugin for ComputePlugin<CM>
where
    CM: ComputeShader,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // ExtractComponentPlugin::<ComputeModel<CM>>::default(),
            ExtractComponentPlugin::<ComputeState<CM::State>>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, extract_components::<CM>)
            .add_systems(
                Render,
                (
                    queue_pipeline::<CM>.in_set(RenderSet::Queue),
                    prepare_bind_group::<CM>.in_set(RenderSet::PrepareBindGroups),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) else {
            return;
        };

        render_app
            .init_resource::<NannouComputePipeline<CM>>()
            .init_resource::<SpecializedComputePipelines<NannouComputePipeline<CM>>>()
            .add_render_graph_node::<ViewNodeRunner<NannouComputeNode<CM>>>(
                Core3d,
                NannouComputeNodeLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::StartMainPass,
                    NannouComputeNodeLabel,
                    Node3d::MainOpaquePass,
                ),
            );
    }
}

fn extract_components<CM>(
    mut commands: Commands,
    mut main_world: ResMut<MainWorld>,
)
    where CM: ComputeShader
{
    let mut query = main_world.query::<(Entity, &ComputeModel<CM>)>();
        let query = query.iter(&main_world);
    for (entity, compute_model) in query {
        info!("Extracting compute model");
        commands.entity(entity).insert(compute_model.clone());
    }
}

fn queue_pipeline<CM>(
    mut commands: Commands,
    pipeline: Res<NannouComputePipeline<CM>>,
    mut pipelines: ResMut<SpecializedComputePipelines<NannouComputePipeline<CM>>>,
    pipeline_cache: Res<PipelineCache>,
    views_q: Query<(Entity, &ComputeState<CM::State>)>,
) where
    CM: ComputeShader,
{
    for (entity, state) in views_q.iter() {
        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            NannouComputePipelineKey {
                shader_entry: CM::shader_entry(&state.0),
            },
        );
        info!("queue pipeline");
        commands
            .entity(entity)
            .insert(ComputePipelineId(pipeline_id));
    }
}

fn prepare_bind_group<CM>(
    mut commands: Commands,
    pipeline: Res<NannouComputePipeline<CM>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    views_q: Query<(Entity, &ComputeModel<CM>)>,
) where
    CM: ComputeShader,
{
    for (view, compute_model) in views_q.iter() {
        let bind_group = compute_model
            .0
            .as_bind_group(
                &pipeline.layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .expect("Failed to create bind group");
        info!("Prepare bind group for view {}", view);
        commands
            .entity(view)
            .insert(ComputeBindGroup(bind_group.bind_group));
    }
}

#[derive(Component, ExtractComponent, Deref, DerefMut, Clone, Default)]
pub struct ComputeState<S: Default + Clone + Send + Sync + 'static>(pub S);

#[derive(Component, ExtractComponent, Clone)]
pub struct ComputeModel<CM: ComputeShader>(pub CM);

#[derive(Component)]
pub struct ComputeBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct ComputeShaderHandle(pub ShaderRef);

#[derive(Component)]
pub struct ComputePipelineId(CachedComputePipelineId);

#[derive(Resource)]
struct NannouComputePipeline<CM>
where
    CM: ComputeShader,
{
    shader: Handle<Shader>,
    layout: BindGroupLayout,
    _compute_model: std::marker::PhantomData<CM>,
}

impl<CM> FromWorld for NannouComputePipeline<CM>
where
    CM: ComputeShader,
{
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();
        let shader = world.resource::<ComputeShaderHandle>();
        let shader = match &shader.0 {
            ShaderRef::Default => panic!("Default shader not supported"),
            ShaderRef::Handle(handle) => handle.clone(),
            ShaderRef::Path(path) => asset_server.load(path),
        };
        NannouComputePipeline {
            shader: shader.clone(),
            layout: CM::bind_group_layout(&render_device),
            _compute_model: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NannouComputePipelineKey {
    pub shader_entry: String,
}

impl<CM> SpecializedComputePipeline for NannouComputePipeline<CM>
where
    CM: ComputeShader,
{
    type Key = NannouComputePipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        info!("Specializing compute pipeline");
        ComputePipelineDescriptor {
            label: None,
            layout: vec![self.layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: self.shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from(key.shader_entry),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct NannouComputeNodeLabel;

pub(crate) struct NannouComputeNode<CM>(std::marker::PhantomData<CM>);

impl<CM> FromWorld for NannouComputeNode<CM>
where
    CM: ComputeShader,
{
    fn from_world(world: &mut World) -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<CM> ViewNode for NannouComputeNode<CM>
where
    CM: ComputeShader,
{
    type ViewQuery = (
        &'static ComputeBindGroup,
        &'static ComputePipelineId,
        &'static ComputeState<CM::State>,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (bind_group, pipeline_id, state): QueryItem<
            'w,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        info!("Running compute node\n\n\n");
        let pipeline = world.resource::<NannouComputePipeline<CM>>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.0) else {
            return Ok(());
        };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &bind_group.0, &[]);
        pass.set_pipeline(pipeline);
        let (x, y, z) = CM::workgroup_size(&state.0);
        pass.dispatch_workgroups(x, y, z);
        Ok(())
    }
}

pub trait ComputeShader: AsBindGroup + Clone + Send + Sync + 'static {
    type State: Default + Clone + Send + Sync + 'static;

    fn compute_shader() -> ShaderRef;

    fn shader_entry(state: &Self::State) -> String {
        "compute".to_string()
    }

    fn workgroup_size(state: &Self::State) -> (u32, u32, u32) {
        (1, 1, 1)
    }
}
