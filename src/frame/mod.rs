//! Items related to the **Frame** type, describing a single frame of graphics for a single window.

use crate::draw::properties::color::IntoRgba;
use crate::vk::{self, DeviceOwned};
use crate::window::SwapchainFramebuffers;
use std::error::Error as StdError;
use std::sync::Arc;
use std::{fmt, ops};

pub mod raw;

pub use self::raw::{AddCommands, RawFrame};

/// A **Frame** to which the user can draw graphics before it is presented to the display.
///
/// **Frame**s are delivered to the user for drawing via the user's **view** function.
///
/// See the **RawFrame** docs for more details on how the implementation works under the hood. The
/// **Frame** type differs in that rather than drawing directly to the swapchain image the user may
/// draw to an intermediary image. There are several advantages of drawing to an intermediary
/// image.
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
    render_pass: Arc<vk::RenderPassAbstract + Send + Sync>,
    // The intermediary image to which the user will draw.
    //
    // The number of multisampling samples may be specified by the user when constructing the
    // window with which the `Frame` is associated.
    pub(crate) intermediary_image: Arc<vk::AttachmentImage>,
    intermediary_image_is_new: bool,
    swapchain_framebuffers: SwapchainFramebuffers,
}

/// Errors that might occur during creation of the `RenderData` for a frame.
#[derive(Debug)]
pub enum RenderDataCreationError {
    RenderPassCreation(vk::RenderPassCreationError),
    ImageCreation(vk::ImageCreationError),
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

        if vk::AttachmentImage::dimensions(&data.intermediary_image) != image_dims {
            let msaa_samples = vk::image::ImageAccess::samples(&data.intermediary_image);
            data.intermediary_image = create_intermediary_image(
                raw_frame.swapchain_image().swapchain().device().clone(),
                image_dims,
                msaa_samples,
                raw_frame.swapchain_image().swapchain().format(),
            )?;
            data.intermediary_image_is_new = true;
        }

        {
            let RenderData {
                ref mut swapchain_framebuffers,
                ref intermediary_image,
                ref render_pass,
                ..
            } = data;

            // Ensure framebuffers are up to date with the frame's swapchain image and render pass.
            swapchain_framebuffers.update(&raw_frame, render_pass.clone(), |builder, image| {
                builder.add(intermediary_image.clone())?.add(image)
            })?;
        }

        Ok(Frame { raw_frame, data })
    }

    /// Called after the user's `view` function returns, this consumes the `Frame`, adds commands
    /// for drawing the `intermediary_image` to the `swapchain_image` and returns the inner
    /// `RenderData` and `RawFrame` so that the `RenderData` may be stored back within the `Window`
    /// and the `RawFrame` may be `finish`ed.
    pub(crate) fn finish(self) -> Result<(RenderData, RawFrame), FrameFinishError> {
        let Frame {
            mut data,
            raw_frame,
        } = self;

        // The framebuffer for the current swapchain image.
        let framebuffer = data.swapchain_framebuffers[raw_frame.swapchain_image_index()].clone();

        // Neither the intermediary image nor swapchain image require clearing upon load.
        let clear_values = vec![vk::ClearValue::None, vk::ClearValue::None];
        let is_secondary = false;

        raw_frame
            .add_commands()
            .begin_render_pass(framebuffer, is_secondary, clear_values)?
            .end_render_pass()
            .expect("failed to add `end_render_pass` command");

        // The intermediary image is no longer "new".
        data.intermediary_image_is_new = false;

        Ok((data, raw_frame))
    }

    /// The image to which all graphics should be drawn this frame.
    ///
    /// After the **view** function returns, this image will be resolved to the swapchain image for
    /// this frame and then that swapchain image will be presented to the screen.
    pub fn image(&self) -> &Arc<vk::AttachmentImage> {
        &self.data.intermediary_image
    }

    /// If the **Frame**'s image is new because it is the first frame or because it was recently
    /// recreated to match the dimensions of the window's swapchain, this will return `true`.
    ///
    /// This is useful for determining whether or not a user's framebuffer might need
    /// reconstruction in the case that it targets the frame's image.
    pub fn image_is_new(&self) -> bool {
        self.raw_frame.nth() == 0 || self.data.intermediary_image_is_new
    }

    /// The color format of the `Frame`'s intermediary image.
    pub fn image_format(&self) -> vk::Format {
        vk::image::ImageAccess::format(self.image())
    }

    /// The number of MSAA samples of the `Frame`'s intermediary image.
    pub fn image_msaa_samples(&self) -> u32 {
        vk::image::ImageAccess::samples(self.image())
    }

    /// Clear the image with the given color.
    pub fn clear<C>(&self, color: C)
    where
        C: IntoRgba<f32>,
    {
        let rgba = color.into_rgba();
        let value = vk::ClearValue::Float([rgba.red, rgba.green, rgba.blue, rgba.alpha]);
        let image = self.data.intermediary_image.clone();
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
        format: vk::Format,
    ) -> Result<Self, RenderDataCreationError> {
        let render_pass = create_render_pass(device.clone(), format, msaa_samples)?;
        let intermediary_image =
            create_intermediary_image(device, dimensions, msaa_samples, format)?;
        let swapchain_framebuffers = Default::default();
        let intermediary_image_is_new = true;
        Ok(RenderData {
            render_pass,
            intermediary_image,
            swapchain_framebuffers,
            intermediary_image_is_new,
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

impl StdError for RenderDataCreationError {
    fn description(&self) -> &str {
        match *self {
            RenderDataCreationError::RenderPassCreation(ref err) => err.description(),
            RenderDataCreationError::ImageCreation(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            RenderDataCreationError::RenderPassCreation(ref err) => Some(err),
            RenderDataCreationError::ImageCreation(ref err) => Some(err),
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

    fn cause(&self) -> Option<&StdError> {
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

    fn cause(&self) -> Option<&StdError> {
        match *self {
            FrameFinishError::BeginRenderPass(ref err) => Some(err),
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

// A function to simplify creating the `vk::AttachmentImage` used as the intermediary image render
// target.
//
// If `msaa_samples` is 0 or 1, a non-multisampled image will be created.
fn create_intermediary_image(
    device: Arc<vk::Device>,
    dimensions: [u32; 2],
    msaa_samples: u32,
    format: vk::Format,
) -> Result<Arc<vk::AttachmentImage>, vk::ImageCreationError> {
    let usage = vk::ImageUsage {
        transfer_source: true,
        transfer_destination: true,
        color_attachment: true,
        //sampled: true,
        ..vk::ImageUsage::none()
    };
    match msaa_samples {
        0 | 1 => vk::AttachmentImage::with_usage(device, dimensions, format, usage),
        _ => vk::AttachmentImage::multisampled_with_usage(
            device,
            dimensions,
            msaa_samples,
            format,
            usage,
        ),
    }
}

// Create the render pass for drawing the intermediary image to the swapchain image.
fn create_render_pass(
    device: Arc<vk::Device>,
    color_format: vk::Format,
    msaa_samples: u32,
) -> Result<Arc<vk::RenderPassAbstract + Send + Sync>, vk::RenderPassCreationError> {
    match msaa_samples {
        // Render pass without multisampling.
        0 | 1 => {
            let rp = vk::single_pass_renderpass!(
                device,
                attachments: {
                    intermediary_color: {
                        load: Load,
                        store: Store,
                        format: color_format,
                        samples: 1,
                    },
                    swapchain_color: {
                        load: DontCare,
                        store: Store,
                        format: color_format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [swapchain_color],
                    depth_stencil: {}
                }
            )?;
            Ok(Arc::new(rp) as Arc<vk::RenderPassAbstract + Send + Sync>)
        }

        // Renderpass with multisampling.
        _ => {
            let rp = vk::single_pass_renderpass!(
                device,
                attachments: {
                    intermediary_color: {
                        load: Load,
                        store: Store,
                        format: color_format,
                        samples: msaa_samples,
                    },
                    swapchain_color: {
                        load: DontCare,
                        store: Store,
                        format: color_format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [intermediary_color],
                    depth_stencil: {}
                    resolve: [swapchain_color],
                }
            )?;
            Ok(Arc::new(rp) as Arc<vk::RenderPassAbstract + Send + Sync>)
        }
    }
}
