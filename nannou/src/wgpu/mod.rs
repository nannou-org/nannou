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
pub mod blend;
mod device_map;
mod render_pass;
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
pub use self::render_pass::{
    Builder as RenderPassBuilder,
    ColorAttachmentDescriptorBuilder as RenderPassColorAttachmentDescriptorBuilder,
};
pub use self::render_pipeline_builder::{RenderPipelineBuilder, VertexDescriptor};
pub use self::sampler_builder::SamplerBuilder;
pub use self::texture::capturer::{
    Capturer as TextureCapturer, Rgba8ReadMapping, Snapshot as TextureSnapshot,
};
pub use self::texture::image::{
    format_from_image_color_type as texture_format_from_image_color_type, BufferImage,
    ImageReadMapping,
};
pub use self::texture::reshaper::Reshaper as TextureReshaper;
pub use self::texture::{
    descriptor_eq as texture_descriptor_eq, extent_3d_eq, format_to_component_type as texture_format_to_component_type,
    format_size_bytes as texture_format_size_bytes, BufferBytes, Builder as TextureBuilder,
    Texture, TextureId, TextureView, TextureViewId, ToTextureView,
};
#[doc(inline)]
pub use wgpu::{
    read_spirv, Adapter, AdapterInfo, AddressMode, BackendBit, BindGroup, BindGroupDescriptor,
    BindGroupLayout, BindGroupLayoutEntry, BindGroupLayoutDescriptor, Binding, BindingResource,
    BindingType, BlendDescriptor, BlendFactor, BlendOperation, Buffer, BufferAddress,
    BufferReadMapping, BufferWriteMapping, BufferAsyncErr, BufferCopyView, BufferDescriptor,
    BufferUsage, Color, ColorStateDescriptor, ColorWrite, CommandBuffer, CommandBufferDescriptor,
    CommandEncoder, CommandEncoderDescriptor, CompareFunction, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, CreateBufferMapped, CullMode, DepthStencilStateDescriptor, Device,
    DeviceDescriptor, Extensions, Extent3d, FilterMode, FrontFace, IndexFormat, InputStepMode,
    Limits, LoadOp, Maintain, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PowerPreference,
    PresentMode, PrimitiveTopology, ProgrammableStageDescriptor, Queue,
    RasterizationStateDescriptor, RenderPass, RenderPassColorAttachmentDescriptor,
    RenderPassDepthStencilAttachmentDescriptor, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerDescriptor, ShaderLocation,
    ShaderModule, ShaderStage, StencilOperation, StencilStateFaceDescriptor, StoreOp, Surface,
    SwapChain, SwapChainDescriptor, SwapChainOutput, Texture as TextureHandle, TextureAspect,
    TextureComponentType, TextureCopyView, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsage, TextureView as TextureViewHandle, TextureViewDescriptor, TextureViewDimension,
    VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat, VertexStateDescriptor,
};

pub fn shader_from_spirv_bytes(device: &wgpu::Device, bytes: &[u8]) -> wgpu::ShaderModule {
    let cursor = std::io::Cursor::new(bytes);
    let vs_spirv = wgpu::read_spirv(cursor).expect("failed to read hard-coded SPIRV");
    device.create_shader_module(&vs_spirv)
}

/// The default power preference used for requesting the WGPU adapter.
pub const DEFAULT_POWER_PREFERENCE: PowerPreference = PowerPreference::HighPerformance;

/// Nannou's default WGPU backend preferences.
pub const DEFAULT_BACKENDS: BackendBit = BackendBit::PRIMARY;

/// The default set of `Extensions` used within the `default_device_descriptor()` function.
pub const DEFAULT_EXTENSIONS: Extensions = Extensions {
    anisotropic_filtering: true,
};

/// Adds a simple render pass command to the given encoder that simply clears the given texture
/// with the given colour.
///
/// The given `texture` must have `TextureUsage::OUTPUT_ATTACHMENT` enabled.
pub fn clear_texture(
    texture: &TextureViewHandle,
    clear_color: Color,
    encoder: &mut CommandEncoder,
) {
    RenderPassBuilder::new()
        .color_attachment(texture, |builder| builder.clear_color(clear_color))
        .begin(encoder);
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
    src_texture: &TextureViewHandle,
    dst_texture: &TextureViewHandle,
    encoder: &mut CommandEncoder,
) {
    RenderPassBuilder::new()
        .color_attachment(src_texture, |color| {
            color
                .load_op(LoadOp::Load)
                .resolve_target_handle(Some(dst_texture))
        })
        .begin(encoder);
}

/// Shorthand for creating the pipeline layout from a slice of bind group layouts.
pub fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::PipelineLayout {
    let descriptor = wgpu::PipelineLayoutDescriptor { bind_group_layouts };
    device.create_pipeline_layout(&descriptor)
}

/// TODO: Remove this once `derive(Clone)` is added to wgpu SamplerDescriptor.
pub fn sampler_descriptor_clone(sampler: &wgpu::SamplerDescriptor) -> wgpu::SamplerDescriptor {
    wgpu::SamplerDescriptor {
        address_mode_u: sampler.address_mode_u,
        address_mode_v: sampler.address_mode_v,
        address_mode_w: sampler.address_mode_w,
        mag_filter: sampler.mag_filter,
        min_filter: sampler.min_filter,
        mipmap_filter: sampler.mipmap_filter,
        lod_min_clamp: sampler.lod_min_clamp,
        lod_max_clamp: sampler.lod_max_clamp,
        compare: sampler.compare,
    }
}

/// The functions within this module use unsafe in order to retrieve their input as a slice of
/// bytes. This is necessary in order to upload data to the GPU via the wgpu
/// `create_buffer_with_data` buffer constructor. This method is unsafe as the type `T` may contain
/// padding which is considered to be uninitialised memory in Rust and may potentially lead to
/// undefined behaviour.
///
/// These should be replaced in the future with something similar to `zerocopy`. Unfortunately, we
/// don't gain much benefit from using `zerocopy` in our case as `zerocopy` provides no way to
/// implement the `AsBytes` trait for generic types (e.g. `Vector*`), even with their type
/// parameters filled (e.g. `Vector2<f32>`). This means we can't derive `AsBytes` for the majority
/// of the types where we need to as `derive(AsBytes)` requires that all fields implement
/// `AsBytes`, and neither our `Vector` types or the palette color types can implement it.
///
/// There is a relatively new crate `bytemuck` which provides traits for this, however these traits
/// are `unsafe` and so we don't gain much benefit in terms of safety, especially for our simple
/// use-case. There is a `zeroable` crate that attempts to derive the `Zeroable` trait from
/// `bytemuck`, however:
/// 1. there not yet any other publicly dependent crates or public discussion around the safety of
///    the provided derives and
/// 2. we would still require implementing `Pod` unsafely.
pub mod bytes {
    pub unsafe fn from_slice<T>(slice: &[T]) -> &[u8]
    where
        T: Copy + Sized,
    {
        let len = slice.len() * std::mem::size_of::<T>();
        let ptr = slice.as_ptr() as *const u8;
        std::slice::from_raw_parts(ptr, len)
    }

    pub unsafe fn from<T>(t: &T) -> &[u8]
    where
        T: Copy + Sized,
    {
        let len = std::mem::size_of::<T>();
        let ptr = t as *const T as *const u8;
        std::slice::from_raw_parts(ptr, len)
    }
}
