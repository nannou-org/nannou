use crate::{
    app::{ModelHolder, RenderFnRes},
    frame::{ExtractedWindowsScaleFactor, Frame},
    prelude::bevy_render::extract_resource::extract_resource,
};
use bevy::{
    app::{App, Plugin},
    camera::NormalizedRenderTarget,
    core_pipeline::schedule::{Core3d, Core3dSystems},
    diagnostic::FrameCount,
    ecs::{
        entity::{ContainsEntity, Entity},
        prelude::Query,
    },
    prelude::{IntoScheduleConfigs, Res},
    render::{
        ExtractSchedule,
        camera::ExtractedCamera,
        renderer::{RenderContext, RenderDevice, ViewQuery},
        sync_world::MainEntity,
        view::{ExtractedWindow, ViewTarget},
    },
    time::Time,
};
use std::ops::Deref;

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

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    extract_resource::<RenderFnRes<M>, bevy::render::RenderApp, ()>,
                    extract_resource::<ModelHolder<M>, bevy::render::RenderApp, ()>,
                ),
            )
            .add_systems(
                Core3d,
                nannou_render_system::<M>.in_set(Core3dSystems::MainPass),
            );
    }
}

pub struct RenderApp {
    elapsed_secs: f32,
    delta_secs: f32,
}

impl RenderApp {
    /// Get the elapsed seconds since startup.
    pub fn time(&self) -> f32 {
        self.elapsed_secs
    }

    /// Get the elapsed seconds since the last frame.
    pub fn time_delta(&self) -> f32 {
        self.delta_secs
    }
}

fn nannou_render_system<M>(
    view: ViewQuery<(&ViewTarget, &ExtractedCamera)>,
    windows: Query<(MainEntity, &ExtractedWindow)>,
    mut ctx: RenderContext,
    render_fn: Option<Res<RenderFnRes<M>>>,
    model: Option<Res<ModelHolder<M>>>,
    render_device: Res<RenderDevice>,
    scale_factors: Res<ExtractedWindowsScaleFactor>,
    time: Res<Time>,
    frame_count: Res<FrameCount>,
) where
    M: Send + Sync + Clone + 'static,
{
    let (Some(render_fn_res), Some(model)) = (render_fn, model) else {
        return;
    };
    let Some(render_fn) = &**render_fn_res else {
        return;
    };

    let (view_target, camera) = view.into_inner();
    let Some(NormalizedRenderTarget::Window(window_ref)) = camera.target else {
        return;
    };
    let window_entity = window_ref.entity();
    let Some(extracted_window) = windows
        .iter()
        .find_map(|(main_entity, window)| (main_entity == window_entity).then_some(window))
    else {
        return;
    };
    let render_app = RenderApp {
        elapsed_secs: time.elapsed_secs(),
        delta_secs: time.delta_secs(),
    };
    let frame = Frame::new(
        &render_device,
        &scale_factors,
        window_entity,
        view_target,
        extracted_window,
        frame_count.0,
        &mut ctx,
    );

    render_fn(&render_app, &model.deref(), frame);
}
