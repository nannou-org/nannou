use crate::prelude::bevy_render::{MainWorld, extract_component::ExtractComponent};
use crate::prelude::{AsBindGroup, CachedPipelineState};
use bevy::{
    core_pipeline::schedule::{Core3d, Core3dSystems},
    ecs::system::StaticSystemParam,
    platform::collections::HashMap,
    prelude::*,
    render::{
        Render, RenderSystems,
        extract_component::ExtractComponentPlugin,
        render_resource::{
            BindGroupLayoutDescriptor, CachedComputePipelineId, ComputePipelineDescriptor,
            PipelineCache, SpecializedComputePipeline, SpecializedComputePipelines,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
    },
    shader::ShaderRef,
};
use std::{borrow::Cow, hash::Hash};
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
                    queue_pipeline::<CM>.in_set(RenderSystems::Queue),
                    prepare_bind_group::<CM>.in_set(RenderSystems::PrepareBindGroups),
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
            .add_systems(
                Core3d,
                nannou_compute_system::<CM>.in_set(Core3dSystems::Prepass),
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
                    // Only signal that `next` is ready - the main world performs the actual
                    // advancement so that `current` and the `ComputeModel` bind group are
                    // updated together. Advancing `current` here would race with component
                    // extraction and leave the dispatched entry point bound to the previous
                    // state's bind group.
                    CachedPipelineState::Ok(_) => {
                        if !state.next_ready {
                            state.next_ready = true;
                        }
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
    pipeline_cache: Res<PipelineCache>,
    views_q: Query<(Entity, &ComputeModel<CM>)>,
    mut bind_group_param: StaticSystemParam<CM::Param>,
) where
    CM: Compute,
{
    for (view, compute_model) in views_q.iter() {
        let bind_group = compute_model
            .0
            .as_bind_group(
                &pipeline.layout_descriptor,
                &render_device,
                &pipeline_cache,
                &mut bind_group_param,
            )
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
    /// Set by `sync_pipeline_cache` in the render world once the pipeline for `next` has
    /// compiled. The main-world compute system reads this to advance `current` to `next`
    /// while it rebuilds the matching bind group, keeping the dispatched entry point and
    /// bind group in sync.
    pub next_ready: bool,
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
    layout_descriptor: BindGroupLayoutDescriptor,
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
            layout_descriptor: CM::bind_group_layout_descriptor(&render_device),
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
            layout: vec![self.layout_descriptor.clone()],
            immediate_size: 0,
            shader: self.shader.clone(),
            shader_defs: vec![],
            entry_point: Some(Cow::from(key.shader_entry)),
            zero_initialize_workgroup_memory: false,
        }
    }
}

fn nannou_compute_system<CM>(
    view: ViewQuery<(&ComputeBindGroup, &ComputeState<CM::State>)>,
    mut ctx: RenderContext,
    pipeline_cache: Res<PipelineCache>,
    pipeline_ids: Res<ComputePipelineIds<CM>>,
) where
    CM: Compute,
{
    let (bind_group, state) = view.into_inner();
    let Some(pipeline_id) = pipeline_ids.get(&state.current) else {
        return;
    };
    let Some(pipeline) = pipeline_cache.get_compute_pipeline(*pipeline_id) else {
        return;
    };
    let mut pass = ctx
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor::default());
    pass.set_bind_group(0, &bind_group.0, &[]);
    pass.set_pipeline(pipeline);
    let (x, y, z) = CM::dispatch_size(&state.current);
    pass.dispatch_workgroups(x, y, z);
}

use bevy::render::render_resource::BindGroup;

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
