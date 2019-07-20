//! Items related to the **Frame** type, describing a single frame of graphics for a single window.

use crate::color::IntoLinSrgba;
use crate::vk::{self, DeviceOwned, DynamicStateBuilder};
use crate::window::SwapchainFramebuffers;
use std::error::Error as StdError;
use std::sync::Arc;
use std::{fmt, ops};

pub mod raw;

pub use self::raw::{AddCommands, RawFrame};

/// The vulkan color format used by the intermediary linear sRGBA image.
///
/// We use a high bit depth format in order to retain as much information as possible when
/// converting from the linear representation to the swapchain format (normally a non-linear
/// representation).
pub const COLOR_FORMAT: vk::Format = vk::Format::R16G16B16A16Unorm;

/// A **Frame** to which the user can draw graphics before it is presented to the display.
///
/// **Frame**s are delivered to the user for drawing via the user's **view** function.
///
/// See the **RawFrame** docs for more details on how the implementation works under the hood. The
/// **Frame** type differs in that rather than drawing directly to the swapchain image the user may
/// draw to an intermediary linear sRGBA image. There are several advantages of drawing to an
/// intermediary image.
pub struct Frame {
    raw_frame: RawFrame,
    data: RenderData,
}

/// A helper type for managing a framebuffer associated with a window's `view` function.
///
/// Creating and maintaining the framebuffer that targets the `Frame`s image can be a tedious task
/// that requires a lot of boilerplate code. This type simplifies the process with a single
/// `update` method that creates or recreates the framebuffer if any of the following conditions
/// are met:
/// - The `update` method is called for the first time.
/// - The `frame.image_is_new()` method indicates that the swapchain or its images have recently
///   been recreated and the framebuffer should be recreated accordingly.
/// - The given render pass is different to that which was used to create the existing framebuffer.
#[derive(Default)]
pub struct ViewFramebufferObject {
    fbo: vk::Fbo,
}

/// Shorthand for the **ViewFramebufferObject** type.
pub type ViewFbo = ViewFramebufferObject;

/// Data necessary for rendering the **Frame**'s `image` to the the `swapchain_image` of the inner
/// raw frame.
pub(crate) struct RenderData {
    intermediary: IntermediaryData,
    swapchain: SwapchainData,
}

// Data related to the intermediary MSAA linear sRGBA image and the non-MSAA linear sRGBA image to
// which the former is resolved.
struct IntermediaryData {
    images: IntermediaryImages,
    images_are_new: bool,
    resolve_render_pass: Option<Arc<dyn vk::RenderPassAbstract + Send + Sync>>,
    resolve_framebuffer: vk::Fbo,
}

// The intermediary MSAA linear sRGBA image and the non-MSAA linear sRGBA image to which the former
// is resolved if necessary. If no MSAA is required, `lin_srgba_msaa` is never created.
struct IntermediaryImages {
    lin_srgba_msaa: Option<Arc<vk::AttachmentImage>>,
    lin_srgba: Arc<vk::AttachmentImage>,
}

// Data related to writing the intermediary linear sRGBA image to the swapchain image.
struct SwapchainData {
    render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
    framebuffers: SwapchainFramebuffers,
    graphics_pipeline: Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>,
    vertex_buffer: Arc<vk::CpuAccessibleBuffer<[Vertex]>>,
    sampler: Arc<vk::Sampler>,
}

// The vertex type used to specify the quad to which the linear sRGBA image is drawn on the
// swapchain image.
#[derive(Copy, Clone, Debug, Default)]
struct Vertex {
    position: [f32; 2],
}

vk::impl_vertex!(Vertex, position);

/// Errors that might occur during creation of the `RenderData` for a frame.
#[derive(Debug)]
pub enum RenderDataCreationError {
    RenderPassCreation(vk::RenderPassCreationError),
    ImageCreation(vk::ImageCreationError),
    GraphicsPipelineCreation(GraphicsPipelineError),
    DeviceMemoryAlloc(vk::memory::DeviceMemoryAllocError),
    SamplerCreation(vk::SamplerCreationError),
}

/// Errors that might occur during creation of the `Frame`.
#[derive(Debug)]
pub enum FrameCreationError {
    ImageCreation(vk::ImageCreationError),
    FramebufferCreation(vk::FramebufferCreationError),
}

/// Errors that might occur during `Frame::finish`.
#[derive(Debug)]
pub enum FrameFinishError {
    BeginRenderPass(vk::command_buffer::BeginRenderPassError),
}

/// Errors that might occur while building the **vk::GraphicsPipeline**.
#[derive(Debug)]
pub enum GraphicsPipelineError {
    Creation(vk::GraphicsPipelineCreationError),
    VertexShaderLoad(vk::OomError),
    FragmentShaderLoad(vk::OomError),
}

impl IntermediaryImages {
    fn dimensions(&self) -> [u32; 2] {
        vk::AttachmentImage::dimensions(&self.lin_srgba)
    }
}

impl Frame {
    /// The default number of multisample anti-aliasing samples used if the window with which the
    /// `Frame` is associated supports it.
    pub const DEFAULT_MSAA_SAMPLES: u32 = 8;

    /// Initialise a new empty frame ready for "drawing".
    pub(crate) fn new_empty(
        raw_frame: RawFrame,
        mut data: RenderData,
    ) -> Result<Self, FrameCreationError> {
        // If the image dimensions differ to that of the swapchain image, recreate it.
        let image_dims = raw_frame.swapchain_image().dimensions();
        if data.intermediary.images.dimensions() != image_dims {
            let msaa_samples = data
                .intermediary
                .images
                .lin_srgba_msaa
                .as_ref()
                .map(|img| vk::image::ImageAccess::samples(img))
                .unwrap_or(1);
            data.intermediary.images = create_intermediary_images(
                raw_frame.swapchain_image().swapchain().device().clone(),
                image_dims,
                msaa_samples,
                COLOR_FORMAT,
            )?;
            data.intermediary.images_are_new = true;
        }

        {
            let RenderData {
                intermediary:
                    IntermediaryData {
                        ref images,
                        ref mut resolve_framebuffer,
                        ref resolve_render_pass,
                        ..
                    },
                ref mut swapchain,
            } = data;

            // Ensure resolve fbo is up to date with the frame's swapchain image and render pass.
            let [w, h] = images.dimensions();
            let dims = [w, h, 1];
            if let Some(msaa_img) = images.lin_srgba_msaa.as_ref() {
                if let Some(rp) = resolve_render_pass.as_ref() {
                    resolve_framebuffer.update(rp.clone(), dims, |builder| {
                        builder.add(msaa_img.clone())?.add(images.lin_srgba.clone())
                    })?;
                }
            }

            // Update the swapchain renderpass fbo.
            let renderpass = swapchain.render_pass.clone();
            swapchain
                .framebuffers
                .update(&raw_frame, renderpass, |builder, image| builder.add(image))?;
        }

        Ok(Frame { raw_frame, data })
    }

    /// Called after the user's `view` function returns, this consumes the `Frame`, adds commands
    /// for drawing the `lin_srgba_msaa` to the `swapchain_image` and returns the inner
    /// `RenderData` and `RawFrame` so that the `RenderData` may be stored back within the `Window`
    /// and the `RawFrame` may be `finish`ed.
    pub(crate) fn finish(self) -> Result<(RenderData, RawFrame), FrameFinishError> {
        let Frame {
            mut data,
            raw_frame,
        } = self;

        // Resolve the MSAA if necessary.
        let clear_values = vec![vk::ClearValue::None, vk::ClearValue::None];
        let is_secondary = false;
        if let Some(fbo) = data.intermediary.resolve_framebuffer.as_ref() {
            raw_frame
                .add_commands()
                .begin_render_pass(fbo.clone(), is_secondary, clear_values)?
                .end_render_pass()
                .expect("failed to add `end_render_pass` command");
        }

        // Draw the linear sRGBA image to the swapchain image.
        //
        // The output of the fragment shader is assumed to be linear by the Vulkan implementation,
        // and it should automatically convert the output to sRGBA for the swapchain if necessary.
        let clear_values = vec![vk::ClearValue::None];
        let image_ix = raw_frame.swapchain_image_index();
        let [w, h] = raw_frame.swapchain_image().dimensions();
        let viewport = vk::ViewportBuilder::new().build([w as _, h as _]);
        let dynamic_state = vk::DynamicState::default().viewports(vec![viewport]);
        let descriptor_set = Arc::new(
            vk::PersistentDescriptorSet::start(data.swapchain.graphics_pipeline.clone(), 0)
                .add_sampled_image(
                    data.intermediary.images.lin_srgba.clone(),
                    data.swapchain.sampler.clone(),
                )
                .expect("failed to add smapled image")
                .build()
                .expect("failed to build descriptor set"),
        );
        raw_frame
            .add_commands()
            .begin_render_pass(
                data.swapchain.framebuffers[image_ix].clone(),
                is_secondary,
                clear_values,
            )?
            .draw(
                data.swapchain.graphics_pipeline.clone(),
                &dynamic_state,
                vec![data.swapchain.vertex_buffer.clone()],
                descriptor_set,
                (),
            )
            .expect("failed to add `draw` command")
            .end_render_pass()
            .expect("failed to add `end_render_pass` command");

        // The intermediary linear sRGBA image is no longer "new".
        data.intermediary.images_are_new = false;

        Ok((data, raw_frame))
    }

    /// The image to which all graphics should be drawn this frame.
    ///
    /// After the **view** function returns, this image will be resolved to the swapchain image for
    /// this frame and then that swapchain image will be presented to the screen.
    pub fn image(&self) -> &Arc<vk::AttachmentImage> {
        match self.data.intermediary.images.lin_srgba_msaa.as_ref() {
            None => &self.data.intermediary.images.lin_srgba,
            Some(msaa_img) => msaa_img,
        }
    }

    /// If the **Frame**'s image is new because it is the first frame or because it was recently
    /// recreated to match the dimensions of the window's swapchain, this will return `true`.
    ///
    /// This is useful for determining whether or not a user's framebuffer might need
    /// reconstruction in the case that it targets the frame's image.
    pub fn image_is_new(&self) -> bool {
        self.raw_frame.nth() == 0 || self.data.intermediary.images_are_new
    }

    /// The color format of the `Frame`'s intermediary linear sRGBA image (equal to
    /// `COLOR_FORMAT`).
    pub fn image_format(&self) -> vk::Format {
        vk::image::ImageAccess::format(self.image())
    }

    /// The number of MSAA samples of the `Frame`'s intermediary linear sRGBA image.
    pub fn image_msaa_samples(&self) -> u32 {
        vk::image::ImageAccess::samples(self.image())
    }

    /// Clear the image with the given color.
    pub fn clear<C>(&self, color: C)
    where
        C: IntoLinSrgba<f32>,
    {
        let lin_srgba = color.into_lin_srgba();
        let (r, g, b, a) = lin_srgba.into_components();
        let value = vk::ClearValue::Float([r, g, b, a]);
        let image = self.image().clone();
        self.add_commands()
            .clear_color_image(image, value)
            .expect("failed to submit `clear_color_image` command");
    }
}

impl ViewFramebufferObject {
    /// Ensure the framebuffer is up to date with the render pass and `frame`'s image.
    pub fn update<R, F, A>(
        &mut self,
        frame: &Frame,
        render_pass: R,
        builder: F,
    ) -> Result<(), vk::FramebufferCreationError>
    where
        R: 'static + vk::RenderPassAbstract + Send + Sync,
        F: FnOnce(
            vk::FramebufferBuilder<R, ()>,
            Arc<vk::AttachmentImage>,
        ) -> vk::FramebufferBuilderResult<R, A>,
        A: 'static + vk::AttachmentsList + Send + Sync,
    {
        let image = frame.image().clone();
        let [w, h] = image.dimensions();
        let dimensions = [w, h, 1];
        self.fbo
            .update(render_pass, dimensions, |b| builder(b, image))
    }
}

impl RenderData {
    /// Initialise the render data.
    ///
    /// Creates an `vk::AttachmentImage` with the given parameters.
    ///
    /// If `msaa_samples` is greater than 1 a `multisampled` image will be created. Otherwise the
    /// a regular non-multisampled image will be created.
    pub(crate) fn new(
        device: Arc<vk::Device>,
        dimensions: [u32; 2],
        msaa_samples: u32,
        swapchain_color_format: vk::Format,
    ) -> Result<Self, RenderDataCreationError> {
        let intermediary_images =
            create_intermediary_images(device.clone(), dimensions, msaa_samples, COLOR_FORMAT)?;
        let resolve_render_pass = create_resolve_render_pass(device.clone(), msaa_samples)?;
        let resolve_framebuffer = Default::default();

        let intermediary = IntermediaryData {
            images: intermediary_images,
            images_are_new: true,
            resolve_render_pass,
            resolve_framebuffer,
        };

        let swapchain_framebuffers = Default::default();
        let swapchain_render_pass =
            create_swapchain_render_pass(device.clone(), swapchain_color_format)?;
        let graphics_pipeline = create_graphics_pipeline(swapchain_render_pass.clone())?;
        let v = |x, y| Vertex { position: [x, y] };
        let tl = v(-1.0, -1.0);
        let bl = v(-1.0, 1.0);
        let tr = v(1.0, -1.0);
        let br = v(1.0, 1.0);
        let vs = [tl, bl, tr, bl, tr, br];
        let vertex_buffer = vk::CpuAccessibleBuffer::from_iter(
            device.clone(),
            vk::BufferUsage::all(),
            vs.iter().cloned(),
        )?;
        let sampler = vk::SamplerBuilder::new().build(device)?;

        let swapchain = SwapchainData {
            render_pass: swapchain_render_pass,
            framebuffers: swapchain_framebuffers,
            graphics_pipeline,
            vertex_buffer,
            sampler,
        };

        Ok(RenderData {
            intermediary,
            swapchain,
        })
    }
}

impl ops::Deref for Frame {
    type Target = RawFrame;
    fn deref(&self) -> &Self::Target {
        &self.raw_frame
    }
}

impl ops::Deref for ViewFramebufferObject {
    type Target = vk::Fbo;
    fn deref(&self) -> &Self::Target {
        &self.fbo
    }
}

impl From<vk::RenderPassCreationError> for RenderDataCreationError {
    fn from(err: vk::RenderPassCreationError) -> Self {
        RenderDataCreationError::RenderPassCreation(err)
    }
}

impl From<vk::ImageCreationError> for RenderDataCreationError {
    fn from(err: vk::ImageCreationError) -> Self {
        RenderDataCreationError::ImageCreation(err)
    }
}

impl From<GraphicsPipelineError> for RenderDataCreationError {
    fn from(err: GraphicsPipelineError) -> Self {
        RenderDataCreationError::GraphicsPipelineCreation(err)
    }
}

impl From<vk::memory::DeviceMemoryAllocError> for RenderDataCreationError {
    fn from(err: vk::memory::DeviceMemoryAllocError) -> Self {
        RenderDataCreationError::DeviceMemoryAlloc(err)
    }
}

impl From<vk::SamplerCreationError> for RenderDataCreationError {
    fn from(err: vk::SamplerCreationError) -> Self {
        RenderDataCreationError::SamplerCreation(err)
    }
}

impl From<vk::ImageCreationError> for FrameCreationError {
    fn from(err: vk::ImageCreationError) -> Self {
        FrameCreationError::ImageCreation(err)
    }
}

impl From<vk::FramebufferCreationError> for FrameCreationError {
    fn from(err: vk::FramebufferCreationError) -> Self {
        FrameCreationError::FramebufferCreation(err)
    }
}

impl From<vk::command_buffer::BeginRenderPassError> for FrameFinishError {
    fn from(err: vk::command_buffer::BeginRenderPassError) -> Self {
        FrameFinishError::BeginRenderPass(err)
    }
}

impl From<vk::GraphicsPipelineCreationError> for GraphicsPipelineError {
    fn from(err: vk::GraphicsPipelineCreationError) -> Self {
        GraphicsPipelineError::Creation(err)
    }
}

impl StdError for RenderDataCreationError {
    fn description(&self) -> &str {
        match *self {
            RenderDataCreationError::RenderPassCreation(ref err) => err.description(),
            RenderDataCreationError::ImageCreation(ref err) => err.description(),
            RenderDataCreationError::GraphicsPipelineCreation(ref err) => err.description(),
            RenderDataCreationError::DeviceMemoryAlloc(ref err) => err.description(),
            RenderDataCreationError::SamplerCreation(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            RenderDataCreationError::RenderPassCreation(ref err) => Some(err),
            RenderDataCreationError::ImageCreation(ref err) => Some(err),
            RenderDataCreationError::GraphicsPipelineCreation(ref err) => Some(err),
            RenderDataCreationError::DeviceMemoryAlloc(ref err) => Some(err),
            RenderDataCreationError::SamplerCreation(ref err) => Some(err),
        }
    }
}

impl StdError for FrameCreationError {
    fn description(&self) -> &str {
        match *self {
            FrameCreationError::ImageCreation(ref err) => err.description(),
            FrameCreationError::FramebufferCreation(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            FrameCreationError::ImageCreation(ref err) => Some(err),
            FrameCreationError::FramebufferCreation(ref err) => Some(err),
        }
    }
}

impl StdError for FrameFinishError {
    fn description(&self) -> &str {
        match *self {
            FrameFinishError::BeginRenderPass(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            FrameFinishError::BeginRenderPass(ref err) => Some(err),
        }
    }
}

impl StdError for GraphicsPipelineError {
    fn description(&self) -> &str {
        match *self {
            GraphicsPipelineError::Creation(ref err) => err.description(),
            GraphicsPipelineError::VertexShaderLoad(ref err) => err.description(),
            GraphicsPipelineError::FragmentShaderLoad(ref err) => err.description(),
        }
    }
}

impl fmt::Display for RenderDataCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for FrameCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for FrameFinishError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Debug for RenderData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RenderData")
    }
}

impl fmt::Display for GraphicsPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

// A function to simplify creating the `vk::AttachmentImage` used as the intermediary image render
// target.
//
// If `msaa_samples` is 0 or 1, a non-multisampled image will be created.
fn create_intermediary_images(
    device: Arc<vk::Device>,
    dimensions: [u32; 2],
    msaa_samples: u32,
    format: vk::Format,
) -> Result<IntermediaryImages, vk::ImageCreationError> {
    let lin_srgba_msaa = match msaa_samples {
        0 | 1 => None,
        _ => {
            let usage = vk::ImageUsage {
                transfer_source: true,
                transfer_destination: true,
                color_attachment: true,
                //sampled: true,
                ..vk::ImageUsage::none()
            };
            Some(vk::AttachmentImage::multisampled_with_usage(
                device.clone(),
                dimensions,
                msaa_samples,
                format,
                usage,
            )?)
        }
    };
    let usage = vk::ImageUsage {
        transfer_source: true,
        transfer_destination: true,
        color_attachment: true,
        sampled: true,
        ..vk::ImageUsage::none()
    };
    let lin_srgba = vk::AttachmentImage::with_usage(device, dimensions, format, usage)?;
    Ok(IntermediaryImages {
        lin_srgba_msaa,
        lin_srgba,
    })
}

// Create the render pass for resolving the MSAA linear sRGB image to the non-MSAA image.
fn create_resolve_render_pass(
    device: Arc<vk::Device>,
    msaa_samples: u32,
) -> Result<Option<Arc<dyn vk::RenderPassAbstract + Send + Sync>>, vk::RenderPassCreationError> {
    match msaa_samples {
        // Render pass without multisampling.
        0 | 1 => Ok(None),

        // Renderpass with multisampling.
        _ => {
            let rp = vk::single_pass_renderpass!(
                device,
                attachments: {
                    lin_srgba_msaa: {
                        load: Load,
                        store: Store,
                        format: COLOR_FORMAT,
                        samples: msaa_samples,
                    },
                    lin_srgba: {
                        load: DontCare,
                        store: Store,
                        format: COLOR_FORMAT,
                        samples: 1,
                    }
                },
                pass: {
                    color: [lin_srgba_msaa],
                    depth_stencil: {}
                    resolve: [lin_srgba],
                }
            )?;
            Ok(Some(
                Arc::new(rp) as Arc<dyn vk::RenderPassAbstract + Send + Sync>
            ))
        }
    }
}

// Create the render pass for drawing the intermediary image to the swapchain image.
fn create_swapchain_render_pass(
    device: Arc<vk::Device>,
    swapchain_color_format: vk::Format,
) -> Result<Arc<dyn vk::RenderPassAbstract + Send + Sync>, vk::RenderPassCreationError> {
    let rp = vk::single_pass_renderpass!(
        device,
        attachments: {
            swapchain_image: {
                load: DontCare,
                store: Store,
                format: swapchain_color_format,
                samples: 1,
            }
        },
        pass: {
            color: [swapchain_image],
            depth_stencil: {}
        }
    )?;
    Ok(Arc::new(rp) as Arc<dyn vk::RenderPassAbstract + Send + Sync>)
}

// A simple graphics pipeline for drawing the linear sRGBA image to the swapchain image.
fn create_graphics_pipeline<R>(
    render_pass: R,
) -> Result<Arc<dyn vk::GraphicsPipelineAbstract + Send + Sync>, GraphicsPipelineError>
where
    R: vk::RenderPassAbstract + Send + Sync + 'static,
{
    let device = render_pass.device().clone();
    let vs = vs::Shader::load(device.clone()).map_err(GraphicsPipelineError::VertexShaderLoad)?;
    let fs = fs::Shader::load(device.clone()).map_err(GraphicsPipelineError::FragmentShaderLoad)?;
    let subpass = vk::Subpass::from(render_pass, 0).expect("no subpass for `id`");
    let pipeline = vk::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(subpass)
        .build(device.clone())?;
    Ok(Arc::new(pipeline))
}

mod vs {
    crate::vk::shaders::shader! {
    ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;

void main() {
    gl_Position = vec4(position * 2.0, 0.0, 1.0);
    tex_coords = position + vec2(0.5);
}"
    }
}

mod fs {
    crate::vk::shaders::shader! {
    ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(tex, tex_coords);
}"
    }
}
