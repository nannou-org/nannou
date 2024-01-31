mod bind_group_builder;
mod render_pass;
mod render_pipeline_builder;
mod sampler_builder;
mod texture;

pub use bind_group_builder::{BindGroupBuilder, BindGroupLayoutBuilder};
pub use render_pass::{
    ColorAttachmentDescriptorBuilder, DepthStencilAttachmentDescriptorBuilder, RenderPassBuilder,
};
pub use render_pipeline_builder::RenderPipelineBuilder;
pub use sampler_builder::SamplerBuilder;
pub use texture::TextureBuilder;

use bevy::prelude::*;
use bevy::render::render_resource as wgpu;

/// Whether or not the sampler descriptor describes a sampler that might perform linear filtering.
///
/// This is used to determine the `filtering` field for the sampler binding type variant which
/// assists wgpu with validation.
pub fn sampler_filtering(desc: &wgpu::SamplerDescriptor) -> bool {
    match (desc.mag_filter, desc.min_filter, desc.mipmap_filter) {
        (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest) => false,
        _ => true,
    }
}

//TODO: Does this need to be a plugin to inject anything into the ECS? Or just utils?
struct NannouWgpuPlugin;

impl Plugin for NannouWgpuPlugin {
    fn build(&self, app: &mut App) {}
}
