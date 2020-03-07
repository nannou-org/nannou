//! Items related to wgpu and its integration in nannou!
//!
//! **WebGPU** is the portable graphics specification that nannou targets allowing us to write code
//! that is both fast and allows us to target a wide range of platforms. **wgpu** is the name of
//! the crate we use that implements this specification.
//!
//! This module re-exports the entire `wgpu` crate along with all of its documentation while also
//! adding some additional items that makes `wgpu` easier to use alongside nannou.
//!
//! Useful links:
//!
//! - An awesome [guide for wgpu-rs](https://sotrh.github.io/learn-wgpu/#what-is-wgpu). Highly
//!   recommended reading if you would like to work more closely with the GPU in nannou!
//! - The [wgpu-rs repository](https://github.com/gfx-rs/wgpu-rs).
//! - The [WebGPU specification](https://gpuweb.github.io/gpuweb/).
//! - WebGPU [on wikipedia](https://en.wikipedia.org/wiki/WebGPU).

mod bind_group_builder;
mod device_map;
mod render_pipeline_builder;
mod sampler_builder;
mod texture;

// Re-export all of `wgpu` along with its documentation.
//
// We do this manually rather than a glob-re-export in order to rename `Texture` to `TextureHandle`
// and have it show up in the documentation properly.
pub use self::bind_group_builder::{
    Builder as BindGroupBuilder, LayoutBuilder as BindGroupLayoutBuilder,
};
pub use self::device_map::{
    ActiveAdapter, AdapterMap, AdapterMapKey, DeviceMap, DeviceMapKey, DeviceQueuePair,
};
pub use self::render_pipeline_builder::{RenderPipelineBuilder, VertexDescriptor};
pub use self::sampler_builder::SamplerBuilder;
pub use self::texture::capturer::{
    Capturer as TextureCapturer, Rgba8AsyncMapping, Snapshot as TextureSnapshot,
};
pub use self::texture::image::{
    format_from_image_color_type as texture_format_from_image_color_type, BufferImage,
    ImageAsyncMapping,
};
pub use self::texture::reshaper::Reshaper as TextureReshaper;
pub use self::texture::{
    descriptor_eq as texture_descriptor_eq, extent_3d_eq,
    format_size_bytes as texture_format_size_bytes, BufferBytes, Builder as TextureBuilder,
    Texture,
};
#[doc(inline)]
pub use wgpu::{
    read_spirv, Adapter, AdapterInfo, AddressMode, BackendBit, BindGroup, BindGroupDescriptor,
    BindGroupLayout, BindGroupLayoutBinding, BindGroupLayoutDescriptor, Binding, BindingResource,
    BindingType, BlendDescriptor, BlendFactor, BlendOperation, Buffer, BufferAddress,
    BufferAsyncMapping, BufferCopyView, BufferDescriptor, BufferMapAsyncResult,
    BufferMapAsyncStatus, BufferUsage, Color, ColorStateDescriptor, ColorWrite, CommandBuffer,
    CommandBufferDescriptor, CommandEncoder, CommandEncoderDescriptor, CompareFunction,
    ComputePass, ComputePipeline, ComputePipelineDescriptor, CreateBufferMapped, CullMode,
    DepthStencilStateDescriptor, Device, DeviceDescriptor, Extensions, Extent3d, FilterMode,
    FrontFace, IndexFormat, InputStepMode, Limits, LoadOp, Origin3d, PipelineLayout,
    PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveTopology,
    ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor, RenderPass,
    RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler,
    SamplerDescriptor, ShaderLocation, ShaderModule, ShaderModuleDescriptor, ShaderStage,
    StencilOperation, StencilStateFaceDescriptor, StoreOp, Surface, SwapChain, SwapChainDescriptor,
    SwapChainOutput, Texture as TextureHandle, TextureAspect, TextureCopyView, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsage, TextureView, TextureViewDescriptor,
    TextureViewDimension, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat,
};

/// The default set of options used to request a `wgpu::Adapter` when creating windows.
pub const DEFAULT_ADAPTER_REQUEST_OPTIONS: RequestAdapterOptions = RequestAdapterOptions {
    power_preference: PowerPreference::HighPerformance,
    backends: BackendBit::PRIMARY,
};

/// The default set of `Extensions` used within the `default_device_descriptor()` function.
pub const DEFAULT_EXTENSIONS: Extensions = Extensions {
    anisotropic_filtering: true,
};

/// Adds a simple render pass command to the given encoder that simply clears the given texture
/// with the given colour.
///
/// The given `texture` must have `TextureUsage::OUTPUT_ATTACHMENT` enabled.
pub fn clear_texture(
    texture: &wgpu::TextureView,
    clear_color: wgpu::Color,
    encoder: &mut wgpu::CommandEncoder,
) {
    let color_attachment = wgpu::RenderPassColorAttachmentDescriptor {
        attachment: texture,
        resolve_target: None,
        load_op: wgpu::LoadOp::Clear,
        store_op: wgpu::StoreOp::Store,
        clear_color,
    };
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[color_attachment],
        depth_stencil_attachment: None,
    };
    let _render_pass = encoder.begin_render_pass(&render_pass_desc);
}

/// The default device descriptor used to instantiate a logical device when creating windows.
pub fn default_device_descriptor() -> DeviceDescriptor {
    let extensions = DEFAULT_EXTENSIONS;
    let limits = Limits::default();
    DeviceDescriptor { extensions, limits }
}

/// Adds a simple render pass command to the given encoder that resolves the given multisampled
/// `src_texture` to the given non-multisampled `dst_texture`.
///
/// Both the `src_texture` and `dst_texture` must have:
///
/// - `TextureUsage::OUTPUT_ATTACHMENT` enabled.
/// - The same dimensions.
/// - The same `TextureFormat`.
pub fn resolve_texture(
    src_texture: &wgpu::TextureView,
    dst_texture: &wgpu::TextureView,
    encoder: &mut wgpu::CommandEncoder,
) {
    let color_attachment = wgpu::RenderPassColorAttachmentDescriptor {
        attachment: src_texture,
        resolve_target: Some(dst_texture),
        load_op: wgpu::LoadOp::Load,
        store_op: wgpu::StoreOp::Store,
        clear_color: wgpu::Color::TRANSPARENT,
    };
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[color_attachment],
        depth_stencil_attachment: None,
    };
    let _render_pass = encoder.begin_render_pass(&render_pass_desc);
}

/// Shorthand for creating the pipeline layout from a slice of bind group layouts.
pub fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::PipelineLayout {
    let descriptor = wgpu::PipelineLayoutDescriptor { bind_group_layouts };
    device.create_pipeline_layout(&descriptor)
}
