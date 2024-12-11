use crate::{
    app::{ModelHolder, RenderFnRes},
    frame::Frame,
    prelude::bevy_render::{
        extract_component::ExtractComponent, extract_resource::extract_resource,
    },
};
pub use bevy::prelude::*;
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    ecs::query::QueryItem,
    render::{
        extract_resource::ExtractResource,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::SpecializedComputePipeline,
        renderer::RenderContext,
        view::{ExtractedView, ExtractedWindows, ViewTarget},
    },
};
use bevy_nannou::prelude::AsBindGroup;
use noise::NoiseFn;
use std::{hash::Hash, ops::Deref};

pub mod compute;

pub struct RenderPlugin<M>(std::marker::PhantomData<M>);

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
        time.elapsed_secs()
    }

    /// Get the elapsed seconds since the last frame.
    pub fn time_delta(&self) -> f32 {
        let time = self.world.resource::<Time>();
        time.delta_secs()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct NannouRenderNodeLabel;

pub(crate) struct NannouRenderNode<M>(std::marker::PhantomData<M>);

impl<M> FromWorld for NannouRenderNode<M>
where
    M: Send + Sync + Clone + 'static,
{
    fn from_world(world: &mut World) -> Self {
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
