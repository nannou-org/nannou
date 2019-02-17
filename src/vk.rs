//! Items related to Vulkan and the Rust API used by Nannou called Vulkano.
//!
//! This module re-exports the entire `vulkano` crate along with all of its documentation while
//! also adding some additional helper types.
//!
//! Individual items from throughout the vulkano crate have been re-exported within this module for
//! ease of access via the `vk::` prefix, removing the need for a lot of boilerplate when coding.
//! However, as a result, the documentation for this module is quite noisey! You can find cleaner
//! information about how the different areas of Vulkan interoperate by checking out the
//! module-level documentation for that area. For example, read about Framebuffers and RenderPasses
//! in the `nannou::vk::framebuffer` module, or read about the CommandBuffer within the
//! `nannou::vk::command_buffer` module.
//!
//! For more information on extensions to the vulkano crate added by nannou, scroll past the
//! "Re-exports" items below.

// Re-export `vulkano` along with its docs under this short-hand `vk` module.
#[doc(inline)]
pub use vulkano::*;

// Re-export type and trait names whose meaning are still obvious outside of their module.
pub use vulkano::{
    buffer::{
        BufferAccess, BufferInner, BufferSlice, BufferUsage, TypedBufferAccess,
        CpuAccessibleBuffer, CpuBufferPool, DeviceLocalBuffer, ImmutableBuffer,
        BufferCreationError, BufferView, BufferViewRef
    },
    buffer::cpu_pool::{
        CpuBufferPoolChunk, CpuBufferPoolSubbuffer,
    },
    command_buffer::{
        AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture,
        DispatchIndirectCommand, DrawIndirectCommand, DynamicState,
        AutoCommandBufferBuilderContextError, ExecuteCommandsError, StateCacherOutcome,
        UpdateBufferError, CommandBuffer,
    },
    descriptor::{
        DescriptorSet, PipelineLayoutAbstract,
    },
    descriptor::descriptor::{
        DescriptorBufferDesc, DescriptorDesc, DescriptorImageDesc, ShaderStages,
        DescriptorDescSupersetError, DescriptorDescTy, DescriptorImageDescArray,
        DescriptorImageDescDimensions, DescriptorType, ShaderStagesSupersetError,
    },
    descriptor::descriptor_set::{
        DescriptorSetsCollection, DescriptorWrite, DescriptorsCount, FixedSizeDescriptorSet,
        FixedSizeDescriptorSetBuilder, FixedSizeDescriptorSetBuilderArray,
        FixedSizeDescriptorSetsPool, PersistentDescriptorSet, PersistentDescriptorSetBuf,
        PersistentDescriptorSetBufView, PersistentDescriptorSetBuilder,
        PersistentDescriptorSetBuilderArray, PersistentDescriptorSetImg,
        PersistentDescriptorSetSampler, StdDescriptorPool, StdDescriptorPoolAlloc,
        UnsafeDescriptorPool, UnsafeDescriptorPoolAllocIter, UnsafeDescriptorSet,
        UnsafeDescriptorSetLayout, DescriptorPoolAllocError, PersistentDescriptorSetBuildError,
        PersistentDescriptorSetError, DescriptorPool, DescriptorPoolAlloc, DescriptorSetDesc,
    },
    descriptor::pipeline_layout::{
        EmptyPipelineDesc, PipelineLayout, PipelineLayoutDescPcRange, PipelineLayoutDescUnion,
        PipelineLayoutSys, RuntimePipelineDesc, PipelineLayoutCreationError,
        PipelineLayoutLimitsError, PipelineLayoutNotSupersetError, RuntimePipelineDescError,
        PipelineLayoutDesc, PipelineLayoutPushConstantsCompatible, PipelineLayoutSetsCompatible,
        PipelineLayoutSuperset,
    },
    device::{
        Device, DeviceExtensions, DeviceOwned, DeviceCreationError, RawDeviceExtensions, Queue,
        QueuesIter,
    },
    format::{
        ClearValue, Format, FormatTy, AcceptsPixels, ClearValuesTuple, FormatDesc,
        PossibleCompressedFormatDesc, PossibleDepthFormatDesc, PossibleDepthStencilFormatDesc,
        PossibleFloatFormatDesc, PossibleFloatOrCompressedFormatDesc, PossibleSintFormatDesc,
        PossibleStencilFormatDesc, PossibleUintFormatDesc, StrongStorage,
    },
    framebuffer::{
        AttachmentDescription, Framebuffer, FramebufferBuilder, FramebufferSys,
        PassDependencyDescription, PassDescription, RenderPass, RenderPassDescAttachments,
        RenderPassDescDependencies, RenderPassDescSubpasses, RenderPassSys, Subpass,
        FramebufferCreationError, IncompatibleRenderPassAttachmentError, LoadOp,
        RenderPassCreationError, StoreOp, SubpassContents, AttachmentsList, FramebufferAbstract,
        RenderPassAbstract, RenderPassCompatible, RenderPassDesc, RenderPassDescClearValues,
        RenderPassSubpassInterface,
    },
    image::{
        AttachmentImage, ImmutableImage, SwapchainImage,
        ImageCreationError, ImageAccess, ImageInner, ImageViewAccess, ImageUsage, StorageImage,
        ImageDimensions, ImageLayout, MipmapsCount,
    },
    image::immutable::{
        ImmutableImageInitialization,
    },
    image::traits::{
        ImageAccessFromUndefinedLayout, AttachmentImageView, ImageClearValue, ImageContent,
    },
    instance::{
        ApplicationInfo, Instance, InstanceExtensions, Limits, PhysicalDevice, PhysicalDevicesIter,
        QueueFamiliesIter, QueueFamily, RawInstanceExtensions, Version, InstanceCreationError,
        PhysicalDeviceType,
    },
    pipeline::{
        ComputePipeline, ComputePipelineSys, GraphicsPipeline, GraphicsPipelineBuilder,
        GraphicsPipelineSys, ComputePipelineCreationError, GraphicsPipelineCreationError,
        ComputePipelineAbstract, GraphicsPipelineAbstract,
    },
    pipeline::blend::{
        AttachmentBlend, Blend, AttachmentsBlend, BlendFactor, BlendOp, LogicOp,
    },
    pipeline::depth_stencil::{
        DepthStencil, Stencil, DepthBounds, StencilOp,
    },
    pipeline::vertex::{
        AttributeInfo, BufferlessDefinition, BufferlessVertices, OneVertexOneInstanceDefinition,
        SingleBufferDefinition, SingleInstanceBufferDefinition, TwoBuffersDefinition,
        VertexMemberInfo, IncompatibleVertexDefinitionError, VertexMemberTy, Vertex,
        VertexDefinition, VertexMember, VertexSource,
    },
    pipeline::viewport::{
        Scissor, Viewport, ViewportsState,
    },
    query::{
        OcclusionQueriesPool, QueryPipelineStatisticFlags, UnsafeQueriesRange, UnsafeQuery,
        UnsafeQueryPool, QueryPoolCreationError, QueryType,
    },
    sampler::{
        Compare as DepthStencilCompare, Sampler, SamplerAddressMode, SamplerCreationError,
        UnnormalizedSamplerAddressMode,
    },
    swapchain::{
        Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreationError,
    },
    sync::{
        Fence, FenceSignalFuture, JoinFuture, NowFuture, Semaphore, SemaphoreSignalFuture,
        GpuFuture,
    },
};
pub use vulkano_shaders as shaders;
pub use vulkano_win as win;

use crate::vk;
use crate::vk::instance::debug::{DebugCallback, DebugCallbackCreationError, Message, MessageTypes};
use crate::vk::instance::loader::{FunctionPointers, Loader};
use std::borrow::Cow;
use std::ops::{self, Range};
use std::panic::RefUnwindSafe;
use std::sync::Arc;

/// The default application name used with the default `ApplicationInfo`.
pub const DEFAULT_APPLICATION_NAME: &'static str = "nannou-app";

/// The default application info
pub const DEFAULT_APPLICATION_INFO: ApplicationInfo<'static> = ApplicationInfo {
    application_name: Some(Cow::Borrowed(DEFAULT_APPLICATION_NAME)),
    application_version: None,
    engine_name: None,
    engine_version: None,
};

/// The **FramebufferObject** or **Fbo** type for easy management of a framebuffer.
///
/// Creating and maintaining a framebuffer and ensuring it is up to date with the given renderpass
/// and images can be a tedious task that requires a lot of boilerplate code. This type simplifies
/// the process with a single `update` method that creates or recreates the framebuffer if any of
/// the following conditions are met:
/// - The `update` method is called for the first time.
/// - The given render pass is different to that which was used to create the existing framebuffer.
/// - The dimensions of the framebuffer don't match the dimensions of the images.
#[derive(Default)]
pub struct FramebufferObject {
    framebuffer: Option<Arc<FramebufferAbstract + Send + Sync>>,
}

/// Shorthand for the **FramebufferObject** type.
pub type Fbo = FramebufferObject;

/// Shorthand for the builder result type expected by the function given to `Fbo::update`.
pub type FramebufferBuilderResult<R, A> =
    Result<FramebufferBuilder<R, A>, FramebufferCreationError>;

/// A builder struct that makes the process of building an instance more modular.
#[derive(Default)]
pub struct InstanceBuilder {
    pub app_info: Option<ApplicationInfo<'static>>,
    pub extensions: Option<InstanceExtensions>,
    pub layers: Vec<String>,
    pub loader: Option<FunctionPointers<Box<dyn Loader + Send + Sync>>>,
}

/// A builder struct that makes the process of building a debug callback more modular.
#[derive(Default)]
pub struct DebugCallbackBuilder {
    pub message_types: Option<MessageTypes>,
    pub user_callback: Option<BoxedUserCallback>,
}

/// A builder struct that makes the process of building a **Sampler** more modular.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SamplerBuilder {
    pub mag_filter: Option<vk::sampler::Filter>,
    pub min_filter: Option<vk::sampler::Filter>,
    pub mipmap_mode: Option<vk::sampler::MipmapMode>,
    pub address_u: Option<SamplerAddressMode>,
    pub address_v: Option<SamplerAddressMode>,
    pub address_w: Option<SamplerAddressMode>,
    pub mip_lod_bias: Option<f32>,
    pub max_anisotropy: Option<f32>,
    pub min_lod: Option<f32>,
    pub max_lod: Option<f32>,
}

/// A builder struct that makes the process of building a **Viewport** more modular.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewportBuilder {
    pub origin: Option<[f32; 2]>,
    pub depth_range: Option<Range<f32>>,
}

// The user vulkan debug callback allocated on the heap to avoid complicated type params.
type BoxedUserCallback = Box<Fn(&Message) + 'static + Send + RefUnwindSafe>;

impl FramebufferObject {
    /// Access the inner framebuffer trait object.
    pub fn inner(&self) -> &Option<Arc<FramebufferAbstract + Send + Sync>> {
        &self.framebuffer
    }

    /// Ensure the framebuffer is up to date with the given dimensions and render pass.
    pub fn update<R, F, A>(
        &mut self,
        render_pass: R,
        dimensions: [u32; 3],
        builder: F,
    ) -> Result<(), FramebufferCreationError>
    where
        R: 'static + vk::framebuffer::RenderPassAbstract + Send + Sync,
        F: FnOnce(FramebufferBuilder<R, ()>) -> FramebufferBuilderResult<R, A>,
        A: 'static + vk::framebuffer::AttachmentsList + Send + Sync,
    {
        let needs_creation = self.framebuffer.is_none()
            || !self.dimensions_match(dimensions)
            || !self.render_passes_match(&render_pass);
        if needs_creation {
            let builder = builder(Framebuffer::start(render_pass))?;
            let fb = builder.build()?;
            self.framebuffer = Some(Arc::new(fb));
        }
        Ok(())
    }

    /// Expects that there is a inner framebuffer object instantiated and returns it.
    ///
    /// **panic!**s if the `update` method has not yet been called.
    ///
    /// This method is shorthand for `fbo.as_ref().expect("inner framebuffer was None").clone()`.
    pub fn expect_inner(&self) -> Arc<FramebufferAbstract + Send + Sync> {
        self.framebuffer
            .as_ref()
            .expect("inner framebuffer was `None` - you must call the `update` method first")
            .clone()
    }

    /// Whether or not the given renderpass matches the framebuffer's render pass.
    pub fn render_passes_match<R>(&self, render_pass: R) -> bool
    where
        R: vk::framebuffer::RenderPassAbstract,
    {
        self.framebuffer
            .as_ref()
            .map(|fb| vk::framebuffer::RenderPassAbstract::inner(fb).internal_object())
            .map(|obj| obj == render_pass.inner().internal_object())
            .unwrap_or(false)
    }

    /// Whether or not the given dimensions match the current dimensions.
    pub fn dimensions_match(&self, dimensions: [u32; 3]) -> bool {
        self.framebuffer
            .as_ref()
            .map(|fb| fb.dimensions() == dimensions)
            .unwrap_or(false)
    }
}

impl InstanceBuilder {
    /// Begin building a vulkano instance.
    pub fn new() -> Self {
        Default::default()
    }

    /// Specify the application info with which the instance should be created.
    pub fn app_info(mut self, app_info: ApplicationInfo<'static>) -> Self {
        self.app_info = Some(app_info);
        self
    }

    /// Specify the exact extensions to enable for the instance.
    pub fn extensions(mut self, extensions: InstanceExtensions) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Add the given extensions to the set of existing extensions within the builder.
    ///
    /// Unlike the `extensions` method, this does not disable pre-existing extensions.
    pub fn add_extensions(mut self, ext: InstanceExtensions) -> Self {
        self.extensions = self.extensions.take()
            .map(|mut e| {
                // TODO: Remove this when `InstanceExtensions::union` gets merged.
                e.khr_surface |= ext.khr_surface;
                e.khr_display |= ext.khr_display;
                e.khr_xlib_surface |= ext.khr_xlib_surface;
                e.khr_xcb_surface |= ext.khr_xcb_surface;
                e.khr_wayland_surface |= ext.khr_wayland_surface;
                e.khr_android_surface |= ext.khr_android_surface;
                e.khr_win32_surface |= ext.khr_win32_surface;
                e.ext_debug_report |= ext.ext_debug_report;
                e.mvk_ios_surface |= ext.mvk_ios_surface;
                e.mvk_macos_surface |= ext.mvk_macos_surface;
                e.mvk_moltenvk |= ext.mvk_moltenvk;
                e.nn_vi_surface |= ext.nn_vi_surface;
                e.ext_swapchain_colorspace |= ext.ext_swapchain_colorspace;
                e.khr_get_physical_device_properties2 |= ext.khr_get_physical_device_properties2;
                e
            })
            .or(Some(ext));
        self
    }

    /// Specify the exact layers to enable for the instance.
    pub fn layers<L>(mut self, layers: L) -> Self
    where
        L: IntoIterator,
        L::Item: Into<String>,
    {
        self.layers = layers.into_iter().map(Into::into).collect();
        self
    }

    /// Extend the existing list of layers with the given layers.
    pub fn add_layers<L>(mut self, layers: L) -> Self
    where
        L: IntoIterator,
        L::Item: Into<String>,
    {
        self.layers.extend(layers.into_iter().map(Into::into));
        self
    }

    /// Build the vulkan instance with the existing parameters.
    pub fn build(self) -> Result<Arc<Instance>, InstanceCreationError> {
        let InstanceBuilder {
            app_info,
            extensions,
            layers,
            loader,
        } = self;

        let app_info = app_info.unwrap_or(DEFAULT_APPLICATION_INFO);
        let extensions = extensions.unwrap_or_else(required_windowing_extensions);
        let layers = layers.iter().map(|s| &s[..]);
        match loader {
            None => Instance::new(Some(&app_info), &extensions, layers),
            Some(loader) => Instance::with_loader(loader, Some(&app_info), &extensions, layers),
        }
    }
}

impl DebugCallbackBuilder {
    /// Begin building a vulkan debug callback.
    pub fn new() -> Self {
        Default::default()
    }

    /// The message types to be emitted to the debug callback.
    ///
    /// If unspecified, nannou will use `MessageTypes::errors_and_warnings`.
    pub fn message_types(mut self, msg_tys: MessageTypes) -> Self {
        self.message_types = Some(msg_tys);
        self
    }

    /// The function that will be called for handling messages.
    ///
    /// If unspecified, nannou will use a function that prints to `stdout` and `stderr`.
    pub fn user_callback<F>(mut self, cb: F) -> Self
    where
        F: Fn(&Message) + 'static + Send + RefUnwindSafe,
    {
        self.user_callback = Some(Box::new(cb) as Box<_>);
        self
    }

    /// Build the debug callback builder for the given vulkan instance.
    pub fn build(
        self,
        instance: &Arc<Instance>,
    ) -> Result<DebugCallback, DebugCallbackCreationError> {
        let DebugCallbackBuilder {
            message_types,
            user_callback,
        } = self;
        let message_types = message_types.unwrap_or_else(|| MessageTypes {
            error: true,
            warning: true,
            performance_warning: true,
            information: true,
            debug: true,
        });
        let user_callback = move |msg: &Message| {
            match user_callback {
                Some(ref cb) => (**cb)(msg),
                None => {
                    let ty = if msg.ty.error {
                        "error"
                    } else if msg.ty.warning {
                        "warning"
                    } else if msg.ty.performance_warning {
                        "performance_warning"
                    } else if msg.ty.information {
                        "information"
                    } else if msg.ty.debug {
                        "debug"
                    } else {
                        println!("[vulkan] <unknown message type>");
                        return;
                    };
                    println!("[vulkan] {} {}: {}", msg.layer_prefix, ty, msg.description);
                }
            };
        };
        DebugCallback::new(instance, message_types, user_callback)
    }
}

impl SamplerBuilder {
    pub const DEFAULT_MAG_FILTER: vk::sampler::Filter = vk::sampler::Filter::Linear;
    pub const DEFAULT_MIN_FILTER: vk::sampler::Filter = vk::sampler::Filter::Linear;
    pub const DEFAULT_MIPMAP_MODE: vk::sampler::MipmapMode = vk::sampler::MipmapMode::Nearest;
    pub const DEFAULT_ADDRESS_U: SamplerAddressMode = SamplerAddressMode::ClampToEdge;
    pub const DEFAULT_ADDRESS_V: SamplerAddressMode = SamplerAddressMode::ClampToEdge;
    pub const DEFAULT_ADDRESS_W: SamplerAddressMode = SamplerAddressMode::ClampToEdge;
    pub const DEFAULT_MIP_LOD_BIAS: f32 = 0.0;
    pub const DEFAULT_MAX_ANISOTROPY: f32 = 1.0;
    pub const DEFAULT_MIN_LOD: f32 = 0.0;
    pub const DEFAULT_MAX_LOD: f32 = 1.0;

    /// Begin building a new vulkan **Sampler**.
    pub fn new() -> Self {
        Self::default()
    }

    /// How the implementation should sample from the image when it is respectively larger than the
    /// original.
    pub fn mag_filter(mut self, filter: vk::sampler::Filter) -> Self {
        self.mag_filter = Some(filter);
        self
    }

    /// How the implementation should sample from the image when it is respectively smaller than
    /// the original.
    pub fn min_filter(mut self, filter: vk::sampler::Filter) -> Self {
        self.min_filter = Some(filter);
        self
    }

    /// How the implementation should choose which mipmap to use.
    pub fn mipmap_mode(mut self, mode: vk::sampler::MipmapMode) -> Self {
        self.mipmap_mode = Some(mode);
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_u(mut self, mode: SamplerAddressMode) -> Self {
        self.address_u = Some(mode);
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_v(mut self, mode: SamplerAddressMode) -> Self {
        self.address_v = Some(mode);
        self
    }

    /// How the implementation should behave when sampling outside of the texture coordinates range
    /// [0.0, 1.0].
    pub fn address_w(mut self, mode: SamplerAddressMode) -> Self {
        self.address_w = Some(mode);
        self
    }

    /// Level of detail bias.
    pub fn mip_lod_bias(mut self, bias: f32) -> Self {
        self.mip_lod_bias = Some(bias);
        self
    }

    /// Must be greater than oro equal to 1.0.
    ///
    /// If greater than 1.0, the implementation will use anisotropic filtering. Using a value
    /// greater than 1.0 requires the sampler_anisotropy feature to be enabled when creating the
    /// device.
    pub fn max_anisotropy(mut self, max: f32) -> Self {
        self.max_anisotropy = Some(max);
        self
    }

    /// The minimum mipmap level to use.
    pub fn min_lod(mut self, lod: f32) -> Self {
        self.min_lod = Some(lod);
        self
    }

    /// The maximum mipmap level to use.
    pub fn max_lod(mut self, lod: f32) -> Self {
        self.max_lod = Some(lod);
        self
    }

    /// Build the sampler with the givenn behaviour.
    pub fn build(self, device: Arc<Device>) -> Result<Arc<Sampler>, SamplerCreationError> {
        let SamplerBuilder {
            mag_filter,
            min_filter,
            mipmap_mode,
            address_u,
            address_v,
            address_w,
            mip_lod_bias,
            max_anisotropy,
            min_lod,
            max_lod,
        } = self;
        Sampler::new(
            device,
            mag_filter.unwrap_or(Self::DEFAULT_MAG_FILTER),
            min_filter.unwrap_or(Self::DEFAULT_MIN_FILTER),
            mipmap_mode.unwrap_or(Self::DEFAULT_MIPMAP_MODE),
            address_u.unwrap_or(Self::DEFAULT_ADDRESS_U),
            address_v.unwrap_or(Self::DEFAULT_ADDRESS_V),
            address_w.unwrap_or(Self::DEFAULT_ADDRESS_W),
            mip_lod_bias.unwrap_or(Self::DEFAULT_MIP_LOD_BIAS),
            max_anisotropy.unwrap_or(Self::DEFAULT_MAX_ANISOTROPY),
            min_lod.unwrap_or(Self::DEFAULT_MIN_LOD),
            max_lod.unwrap_or(Self::DEFAULT_MAX_LOD),
        )
    }
}

impl ViewportBuilder {
    pub const DEFAULT_ORIGIN: [f32; 2] = [0.0; 2];
    pub const DEFAULT_DEPTH_RANGE: Range<f32> = 0.0..1.0;

    /// Begin building a new **Viewport**.
    pub fn new() -> Self {
        Self::default()
    }

    /// Coordinates in pixels of the top-left hand corner of the viewport.
    ///
    /// By default this is `ViewportDefault::DEFAULT_ORIGIN`.
    pub fn origin(mut self, origin: [f32; 2]) -> Self {
        self.origin = Some(origin);
        self
    }

    /// Minimum and maximum values of the depth.
    ///
    /// The values `0.0` to `1.0` of each vertex's Z coordinate will be mapped to this
    /// `depth_range` before being compared to the existing depth value.
    ///
    /// This is equivalents to `glDepthRange` in OpenGL, except that OpenGL uses the Z coordinate
    /// range from `-1.0` to `1.0` instead.
    ///
    /// By default this is `ViewportDefault::DEFAULT_DEPTH_RANGE`.
    pub fn depth_range(mut self, range: Range<f32>) -> Self {
        self.depth_range = Some(range);
        self
    }

    /// Construct the viewport with its dimensions in pixels.
    pub fn build(self, dimensions: [f32; 2]) -> Viewport {
        let ViewportBuilder {
            origin,
            depth_range,
        } = self;
        Viewport {
            origin: origin.unwrap_or(Self::DEFAULT_ORIGIN),
            depth_range: depth_range.unwrap_or(Self::DEFAULT_DEPTH_RANGE),
            dimensions,
        }
    }
}

impl ops::Deref for FramebufferObject {
    type Target = Option<Arc<FramebufferAbstract + Send + Sync>>;
    fn deref(&self) -> &Self::Target {
        &self.framebuffer
    }
}

/// The default set of required extensions used by Nannou.
///
/// This is the same as calling `vk::win::required_extensions()`.
pub fn required_windowing_extensions() -> InstanceExtensions {
    vulkano_win::required_extensions()
}

/// Whether or not the format is sRGB.
pub fn format_is_srgb(format: Format) -> bool {
    use vk::format::Format::*;
    match format {
        R8Srgb |
        R8G8Srgb |
        R8G8B8Srgb |
        B8G8R8Srgb |
        R8G8B8A8Srgb |
        B8G8R8A8Srgb |
        A8B8G8R8SrgbPack32 |
        BC1_RGBSrgbBlock |
        BC1_RGBASrgbBlock |
        BC2SrgbBlock |
        BC3SrgbBlock |
        BC7SrgbBlock |
        ETC2_R8G8B8SrgbBlock |
        ETC2_R8G8B8A1SrgbBlock |
        ETC2_R8G8B8A8SrgbBlock |
        ASTC_4x4SrgbBlock |
        ASTC_5x4SrgbBlock |
        ASTC_5x5SrgbBlock |
        ASTC_6x5SrgbBlock |
        ASTC_6x6SrgbBlock |
        ASTC_8x5SrgbBlock |
        ASTC_8x6SrgbBlock |
        ASTC_8x8SrgbBlock |
        ASTC_10x5SrgbBlock |
        ASTC_10x6SrgbBlock |
        ASTC_10x8SrgbBlock |
        ASTC_10x10SrgbBlock |
        ASTC_12x10SrgbBlock |
        ASTC_12x12SrgbBlock => true,
        _ => false,
    }
}

/// Given some target MSAA samples, limit it by the capabilities of the given `physical_device`.
///
/// This is useful for attempting a specific multisampling sample count but falling back to a
/// supported count in the case that the desired count is unsupported.
///
/// Specifically, this function limits the given `target_msaa_samples` to the minimum of the color
/// and depth sample count limits.
pub fn msaa_samples_limited(physical_device: &PhysicalDevice, target_msaa_samples: u32) -> u32 {
    let color_limit = physical_device.limits().framebuffer_color_sample_counts();
    let depth_limit = physical_device.limits().framebuffer_depth_sample_counts();
    let msaa_limit = std::cmp::min(color_limit, depth_limit);
    std::cmp::min(msaa_limit, target_msaa_samples)
}

#[cfg(all(target_os = "macos", not(test)))]
pub fn check_moltenvk(
    vulkan_builder: InstanceBuilder,
    settings: Option<moltenvk_deps::Install>,
) -> InstanceBuilder {
    let settings = match settings {
        Some(s) => s,
        None => Default::default(),
    };
    let path = match moltenvk_deps::check_or_install(settings) {
        Err(moltenvk_deps::Error::ResetEnvVars(p)) => Some(p),
        Err(moltenvk_deps::Error::NonDefaultDir) => None,
        Err(moltenvk_deps::Error::ChoseNotToInstall) => panic!("Moltenvk is required for Nannou on MacOS"),
        Err(e) => panic!("Moltenvk installation failed {:?}", e),
        Ok(p) => Some(p),
    };
    let loader = path.map(|p| {
        unsafe { DynamicLibraryLoader::new(p) }
    });
    match loader {
        Some(Ok(l)) => {
            let loader: FunctionPointers<Box<(dyn Loader + Send + Sync + 'static)>> = FunctionPointers::new(Box::new(l));
            let required_extensions = required_extensions_with_loader(&loader);
            vulkan_builder.extensions(required_extensions)
            .add_loader(loader)
        },
        _ => vulkan_builder,
    }
}

pub fn required_extensions_with_loader<L>(ptrs: &FunctionPointers<L>)
    -> InstanceExtensions
    where L: Loader
{
    let ideal = InstanceExtensions {
        khr_surface: true,
        khr_xlib_surface: true,
        khr_xcb_surface: true,
        khr_wayland_surface: true,
        khr_android_surface: true,
        khr_win32_surface: true,
        mvk_ios_surface: true,
        mvk_macos_surface: true,
        ..InstanceExtensions::none()
    };

    match InstanceExtensions::supported_by_core_with_loader(ptrs) {
        Ok(supported) => supported.intersection(&ideal),
        Err(_) => InstanceExtensions::none(),
    }
}
