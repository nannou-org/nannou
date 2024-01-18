use bevy::asset::load_internal_asset;
use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::core_3d::CORE_3D;
use bevy::prelude::*;
use bevy::render::{Extract, render_resource as wgpu, RenderSet};
use bevy::render::{Render, RenderApp};
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::render_asset::RenderAsset;
use bevy::render::render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner};
use bevy::render::render_phase::{
    AddRenderCommand, PhaseItem, RenderCommand,
};
use bevy::render::render_resource::{
    ShaderType, SpecializedRenderPipeline,
    SpecializedRenderPipelines,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ViewUniforms;

use mesh::ViewMesh;
use pipeline::queue_pipelines;

use crate::pipeline::{NannouPipeline, NannouViewNode};

pub mod mesh;
mod pipeline;
mod text;
// mod reshaper;

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

        app
            .add_plugins(ExtractComponentPlugin::<ViewMesh>::default());


        println!("NannouRenderPlugin::build");
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<SpecializedRenderPipelines<NannouPipeline>>()
            .add_systems(
                Render,
                prepare_view_uniform.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                queue_pipelines
                    .in_set(RenderSet::PrepareAssets)
            )
            .add_render_graph_node::<ViewNodeRunner<NannouViewNode>>(
                core_3d::graph::NAME,
                NannouViewNode::NAME,
            )
            .add_render_graph_edges(
            CORE_3D,
            &[
                core_3d::graph::node::MAIN_TRANSPARENT_PASS,
                NannouViewNode::NAME,
                core_3d::graph::node::END_MAIN_PASS,
            ],
        );

        bevy_mod_debugdump::print_render_graph(app);
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<NannouPipeline>();
    }
}

fn prepare_view_uniform(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipline: Res<NannouPipeline>,
    view_uniforms: Res<ViewUniforms>,
) {
    if let Some(binding) = view_uniforms.uniforms.binding() {
        commands.insert_resource(ViewUniformBindGroup::new(
            &render_device,
            &pipline.view_bind_group_layout,
            binding,
        ));
    }
}
#[derive(Resource)]
struct ViewUniformBindGroup {
    bind_group: wgpu::BindGroup,
}

impl ViewUniformBindGroup {
    fn new(
        device: &RenderDevice,
        layout: &wgpu::BindGroupLayout,
        binding: wgpu::BindingResource,
    ) -> ViewUniformBindGroup {
        let bind_group = bevy_nannou_wgpu::BindGroupBuilder::new()
            .binding(binding)
            .build(device, layout);

        ViewUniformBindGroup { bind_group }
    }
}

/// A top-level indicator of whether or not
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum VertexMode {
    /// Use the color values and ignore the texture coordinates.
    Color = 0,
    /// Use the texture color and ignore the color values.
    Texture = 1,
    /// A special mode used by the text primitive.
    ///
    /// Uses the color values, but multiplies the alpha by the glyph cache texture's red value.
    Text = 2,
}
