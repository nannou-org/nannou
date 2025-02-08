use crate::app::{ModelHolder, RenderFnRes};
use crate::frame::Frame;
use crate::prelude::bevy_render::extract_component::ExtractComponent;
use crate::prelude::bevy_render::extract_resource::extract_resource;
use crate::prelude::bevy_render::{Extract, MainWorld};
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::ecs::entity::{EntityHash, EntityHashMap};
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
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
use bevy::utils::HashMap;
use bevy_nannou::prelude::{
    AsBindGroup, CachedPipelineState, ShaderStages, StorageTextureAccess, TextureFormat,
};
use noise::NoiseFn;
use std::borrow::Cow;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use wgpu::ComputePassDescriptor;

pub(crate) struct ComputePlugin<CM: Compute>(std::marker::PhantomData<CM>);

impl<CM: Compute> Default for ComputePlugin<CM> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<CM> Plugin for ComputePlugin<CM>
where
    CM: Compute,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<ComputeModel<CM>>::default(),
            ExtractComponentPlugin::<ComputeState<CM::State>>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, sync_pipeline_cache::<CM>)
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
            .insert_resource(ComputePipelineIds::<CM>(HashMap::default()))
            .init_resource::<NannouComputePipeline<CM>>()
            .init_resource::<SpecializedComputePipelines<NannouComputePipeline<CM>>>()
            .add_render_graph_node::<ViewNodeRunner<NannouComputeNode<CM>>>(
                Core3d,
                NannouComputeNodeLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::EndPrepasses,
                    NannouComputeNodeLabel,
                    Node3d::StartMainPass,
                ),
            );
    }
}

fn sync_pipeline_cache<CM>(
    mut main_world: ResMut<MainWorld>,
    pipelines: Res<PipelineCache>,
    pipeline_ids: Res<ComputePipelineIds<CM>>,
) where
    CM: Compute,
{
    let mut states_q = main_world.query::<&mut ComputeState<CM::State>>();
    for mut state in states_q.iter_mut(&mut main_world) {
        if let Some(next_state) = &state.next {
            if let Some(id) = pipeline_ids.get(next_state) {
                match pipelines.get_compute_pipeline_state(*id) {
                    CachedPipelineState::Queued => {}
                    CachedPipelineState::Creating(_) => {}
                    CachedPipelineState::Ok(_) => {
                        state.current = next_state.clone();
                        state.next = None;
                    }
                    CachedPipelineState::Err(e) => {
                        error!("Failed to create pipeline {:?}", e);
                    }
                };
            }
        }
    }
}

fn queue_pipeline<CM>(
    pipeline: Res<NannouComputePipeline<CM>>,
    mut pipelines: ResMut<SpecializedComputePipelines<NannouComputePipeline<CM>>>,
    pipeline_cache: Res<PipelineCache>,
    mut pipeline_ids: ResMut<ComputePipelineIds<CM>>,
    views_q: Query<&ComputeState<CM::State>>,
) where
    CM: Compute,
{
    for state in views_q.iter() {
        if !pipeline_ids.contains_key(&state.current) {
            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &pipeline,
                NannouComputePipelineKey {
                    shader_entry: CM::entry(&state.current),
                },
            );
            pipeline_ids.insert(state.current.clone(), pipeline_id);
        }

        if let Some(next) = &state.next {
            if !pipeline_ids.contains_key(next) {
                let pipeline_id = pipelines.specialize(
                    &pipeline_cache,
                    &pipeline,
                    NannouComputePipelineKey {
                        shader_entry: CM::entry(next),
                    },
                );
                pipeline_ids.insert(next.clone(), pipeline_id);
            }
        }
    }
}

fn prepare_bind_group<CM>(
    mut commands: Commands,
    pipeline: Res<NannouComputePipeline<CM>>,
    render_device: Res<RenderDevice>,
    views_q: Query<(Entity, &ComputeModel<CM>)>,
    mut bind_group_param: StaticSystemParam<CM::Param>,
) where
    CM: Compute,
{
    for (view, compute_model) in views_q.iter() {
        let bind_group = compute_model
            .0
            .as_bind_group(&pipeline.layout, &render_device, &mut bind_group_param)
            .expect("Failed to create bind group");
        commands
            .entity(view)
            .insert(ComputeBindGroup(bind_group.bind_group));
    }
}

#[derive(Component, ExtractComponent, Clone, Default)]
pub(crate) struct ComputeState<S: Default + Clone + Send + Sync + 'static> {
    pub current: S,
    pub next: Option<S>,
}

#[derive(Resource, Deref, DerefMut)]
struct ComputePipelineIds<CM: Compute>(HashMap<CM::State, CachedComputePipelineId>);

#[derive(Component, ExtractComponent, Clone)]
pub(crate) struct ComputeModel<CM: Compute>(pub CM);

#[derive(Component)]
struct ComputeBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct ComputeShaderHandle(pub ShaderRef);

#[derive(Resource)]
struct NannouComputePipeline<CM>
where
    CM: Compute,
{
    shader: Handle<Shader>,
    layout: BindGroupLayout,
    _compute_model: std::marker::PhantomData<CM>,
}

impl<CM> FromWorld for NannouComputePipeline<CM>
where
    CM: Compute,
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
struct NannouComputePipelineKey {
    pub shader_entry: &'static str,
}

impl<CM> SpecializedComputePipeline for NannouComputePipeline<CM>
where
    CM: Compute,
{
    type Key = NannouComputePipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("NannouComputePipeline".into()),
            layout: vec![self.layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: self.shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from(key.shader_entry),
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct NannouComputeNodeLabel;

struct NannouComputeNode<CM>(std::marker::PhantomData<CM>);

impl<CM> FromWorld for NannouComputeNode<CM>
where
    CM: Compute,
{
    fn from_world(_world: &mut World) -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<CM> ViewNode for NannouComputeNode<CM>
where
    CM: Compute,
{
    type ViewQuery = (&'static ComputeBindGroup, &'static ComputeState<CM::State>);

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (bind_group, state): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_ids = world.resource::<ComputePipelineIds<CM>>();
        let Some(pipeline_id) = pipeline_ids.get(&state.current) else {
            return Ok(());
        };
        let Some(pipeline) = pipeline_cache.get_compute_pipeline(*pipeline_id) else {
            return Ok(());
        };
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_bind_group(0, &bind_group.0, &[]);
        pass.set_pipeline(pipeline);
        let (x, y, z) = CM::dispatch_size(&state.current);
        pass.dispatch_workgroups(x, y, z);
        Ok(())
    }
}

pub trait Compute: AsBindGroup + Clone + Send + Sync + 'static {
    type State: Default + Eq + PartialEq + Hash + Clone + Send + Sync + 'static;

    /// The shader to use for this compute model.
    fn shader() -> ShaderRef;

    /// The entry point for the compute shader as derived from the state. This can be
    /// used in combination with state to advance a compute shader through a series of
    /// stages.
    fn entry(_state: &Self::State) -> &'static str {
        "main"
    }

    /// The size used to dispatch the compute pass.
    fn dispatch_size(_state: &Self::State) -> (u32, u32, u32) {
        (1, 1, 1)
    }
}
