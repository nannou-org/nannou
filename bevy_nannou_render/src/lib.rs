use bevy::asset::load_internal_asset;
use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::core_3d::CORE_3D;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::render_asset::RenderAsset;
use bevy::render::render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner};
use bevy::render::render_resource::{
    CachedRenderPipelineId, ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ViewUniforms;
use bevy::render::RenderSet::Prepare;
use bevy::render::{render_resource as wgpu, RenderSet};
use bevy::render::{Render, RenderApp};
use bevy_nannou_draw::draw::render::Scissor;

use crate::pipeline::{NannouPipeline, NannouViewNode, TextureBindGroupCache};

mod pipeline;

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

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<SpecializedRenderPipelines<NannouPipeline>>()
            .init_resource::<TextureBindGroupCache>()
            .add_systems(
                Render,
                (
                    prepare_view_mesh.in_set(RenderSet::PrepareResources),
                    prepare_texture_bind_groups.in_set(RenderSet::PrepareBindGroups),
                    prepare_view_uniform.in_set(RenderSet::PrepareBindGroups),
                ),
            )
            // Register the NannouViewNode with the render graph
            // The node runs at the last stage of the main 3d pass
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
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<NannouPipeline>();
    }
}

// Prepare our mesh for rendering
fn prepare_view_mesh(commands: Commands) {
    // TODO: process the extracted draw components
}

// Prepare our uniform bind group from Bevy's view uniforms
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

// Prepare user uploaded textures for rendering
fn prepare_texture_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_bind_group_cache: ResMut<TextureBindGroupCache>,
) {
    // TODO: draw will add the texture as a component that
    // will be extracted here and added to the cache
}

// Resource wrapper for our view uniform bind group
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

#[derive(Component, Deref, DerefMut)]
pub struct ViewMesh(bevy_nannou_draw::draw::Mesh);
/// Commands that map to wgpu encodable commands.
#[derive(Debug, Clone)]
enum RenderCommand {
    /// Change pipeline for the new blend mode and topology.
    SetPipeline(CachedRenderPipelineId),
    /// Change bind group for a new image.
    SetBindGroup(Handle<Image>),
    /// Set the rectangular scissor.
    SetScissor(Scissor),
    /// Draw the given vertex range.
    DrawIndexed {
        start_vertex: i32,
        index_range: std::ops::Range<u32>,
    },
}

#[derive(Component)]
pub struct ViewRenderCommands {
    commands: Vec<RenderCommand>,
}
