//! The nannou [**Window**](./struct.Window.html) API. Create a new window via `.app.new_window()`.
//! This produces a [**Builder**](./struct.Builder.html) which can be used to build a window.

use crate::app::LoopMode;
use crate::event::{
    Key, MouseButton, MouseScrollDelta, TouchEvent, TouchPhase, TouchpadPressure, WindowEvent,
};
use crate::frame::{self, Frame, RawFrame};
use crate::geom;
use crate::geom::{Point2, Vector2};
use crate::vk::{self, win::VkSurfaceBuild, VulkanObject};
use crate::App;
use std::any::Any;
use std::error::Error as StdError;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::{cmp, env, fmt, ops};
use winit::dpi::LogicalSize;
use winit::{self, MonitorId, MouseCursor};

pub use winit::WindowId as Id;

/// The default dimensions used for a window in the case that none are specified.
pub const DEFAULT_DIMENSIONS: LogicalSize = LogicalSize {
    width: 1024.0,
    height: 768.0,
};

/// A context for building a window.
pub struct Builder<'app> {
    app: &'app App,
    vk_physical_device: Option<vk::PhysicalDevice<'app>>,
    vk_device_extensions: Option<vk::DeviceExtensions>,
    vk_device_queue: Option<Arc<vk::Queue>>,
    window: winit::WindowBuilder,
    title_was_set: bool,
    swapchain_builder: SwapchainBuilder,
    user_functions: UserFunctions,
    msaa_samples: Option<u32>,
}

/// For storing all user functions within the window.
#[derive(Debug, Default)]
pub(crate) struct UserFunctions {
    pub(crate) view: Option<View>,
    pub(crate) event: Option<EventFnAny>,
    pub(crate) raw_event: Option<RawEventFnAny>,
    pub(crate) key_pressed: Option<KeyPressedFnAny>,
    pub(crate) key_released: Option<KeyReleasedFnAny>,
    pub(crate) mouse_moved: Option<MouseMovedFnAny>,
    pub(crate) mouse_pressed: Option<MousePressedFnAny>,
    pub(crate) mouse_released: Option<MouseReleasedFnAny>,
    pub(crate) mouse_entered: Option<MouseEnteredFnAny>,
    pub(crate) mouse_exited: Option<MouseExitedFnAny>,
    pub(crate) mouse_wheel: Option<MouseWheelFnAny>,
    pub(crate) moved: Option<MovedFnAny>,
    pub(crate) resized: Option<ResizedFnAny>,
    pub(crate) touch: Option<TouchFnAny>,
    pub(crate) touchpad_pressure: Option<TouchpadPressureFnAny>,
    pub(crate) hovered_file: Option<HoveredFileFnAny>,
    pub(crate) hovered_file_cancelled: Option<HoveredFileCancelledFnAny>,
    pub(crate) dropped_file: Option<DroppedFileFnAny>,
    pub(crate) focused: Option<FocusedFnAny>,
    pub(crate) unfocused: Option<UnfocusedFnAny>,
    pub(crate) closed: Option<ClosedFnAny>,
}

/// The user function type for drawing their model to the surface of a single window.
pub type ViewFn<Model> = fn(&App, &Model, &Frame);

/// The user function type for drawing their model to the surface of a single window.
///
/// Unlike the `ViewFn`, the `RawViewFn` is designed for drawing directly to a window's swapchain
/// images rather than to a convenient intermediary image.
pub type RawViewFn<Model> = fn(&App, &Model, &RawFrame);

/// The same as `ViewFn`, but provides no user model to draw from.
///
/// Useful for simple, stateless sketching.
pub type SketchFn = fn(&App, &Frame);

/// The user's view function, whether with a model or without one.
#[derive(Clone)]
pub(crate) enum View {
    WithModel(ViewFnAny),
    WithModelRaw(RawViewFnAny),
    Sketch(SketchFn),
}

/// A function for processing raw winit window events.
pub type RawEventFn<Model> = fn(&App, &mut Model, winit::WindowEvent);

/// A function for processing window events.
pub type EventFn<Model> = fn(&App, &mut Model, WindowEvent);

/// A function for processing key press events.
pub type KeyPressedFn<Model> = fn(&App, &mut Model, Key);

/// A function for processing key release events.
pub type KeyReleasedFn<Model> = fn(&App, &mut Model, Key);

/// A function for processing mouse moved events.
pub type MouseMovedFn<Model> = fn(&App, &mut Model, Point2);

/// A function for processing mouse pressed events.
pub type MousePressedFn<Model> = fn(&App, &mut Model, MouseButton);

/// A function for processing mouse released events.
pub type MouseReleasedFn<Model> = fn(&App, &mut Model, MouseButton);

/// A function for processing mouse entered events.
pub type MouseEnteredFn<Model> = fn(&App, &mut Model);

/// A function for processing mouse exited events.
pub type MouseExitedFn<Model> = fn(&App, &mut Model);

/// A function for processing mouse wheel events.
pub type MouseWheelFn<Model> = fn(&App, &mut Model, MouseScrollDelta, TouchPhase);

/// A function for processing window moved events.
pub type MovedFn<Model> = fn(&App, &mut Model, Vector2);

/// A function for processing window resized events.
pub type ResizedFn<Model> = fn(&App, &mut Model, Vector2);

/// A function for processing touch events.
pub type TouchFn<Model> = fn(&App, &mut Model, TouchEvent);

/// A function for processing touchpad pressure events.
pub type TouchpadPressureFn<Model> = fn(&App, &mut Model, TouchpadPressure);

/// A function for processing hovered file events.
pub type HoveredFileFn<Model> = fn(&App, &mut Model, PathBuf);

/// A function for processing hovered file cancelled events.
pub type HoveredFileCancelledFn<Model> = fn(&App, &mut Model);

/// A function for processing dropped file events.
pub type DroppedFileFn<Model> = fn(&App, &mut Model, PathBuf);

/// A function for processing window focused events.
pub type FocusedFn<Model> = fn(&App, &mut Model);

/// A function for processing window unfocused events.
pub type UnfocusedFn<Model> = fn(&App, &mut Model);

/// A function for processing window closed events.
pub type ClosedFn<Model> = fn(&App, &mut Model);

// A macro for generating a handle to a function that can be stored within the Window without
// requiring a type param. $TFn is the function pointer type that will be wrapped by $TFnAny.
macro_rules! fn_any {
    ($TFn:ident<M>, $TFnAny:ident) => {
        // A handle to a function that can be stored without requiring a type param.
        #[derive(Clone)]
        pub(crate) struct $TFnAny {
            fn_ptr: Arc<dyn Any>,
        }

        impl $TFnAny {
            // Create the `$TFnAny` from a view function pointer.
            pub fn from_fn_ptr<M>(fn_ptr: $TFn<M>) -> Self
            where
                M: 'static,
            {
                let fn_ptr = Arc::new(fn_ptr) as Arc<dyn Any>;
                $TFnAny { fn_ptr }
            }

            // Retrieve the view function pointer from the `$TFnAny`.
            pub fn to_fn_ptr<M>(&self) -> Option<&$TFn<M>>
            where
                M: 'static,
            {
                self.fn_ptr.downcast_ref::<$TFn<M>>()
            }
        }

        impl fmt::Debug for $TFnAny {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", stringify!($TFnAny))
            }
        }
    };
}

fn_any!(ViewFn<M>, ViewFnAny);
fn_any!(RawViewFn<M>, RawViewFnAny);
fn_any!(EventFn<M>, EventFnAny);
fn_any!(RawEventFn<M>, RawEventFnAny);
fn_any!(KeyPressedFn<M>, KeyPressedFnAny);
fn_any!(KeyReleasedFn<M>, KeyReleasedFnAny);
fn_any!(MouseMovedFn<M>, MouseMovedFnAny);
fn_any!(MousePressedFn<M>, MousePressedFnAny);
fn_any!(MouseReleasedFn<M>, MouseReleasedFnAny);
fn_any!(MouseEnteredFn<M>, MouseEnteredFnAny);
fn_any!(MouseExitedFn<M>, MouseExitedFnAny);
fn_any!(MouseWheelFn<M>, MouseWheelFnAny);
fn_any!(MovedFn<M>, MovedFnAny);
fn_any!(ResizedFn<M>, ResizedFnAny);
fn_any!(TouchFn<M>, TouchFnAny);
fn_any!(TouchpadPressureFn<M>, TouchpadPressureFnAny);
fn_any!(HoveredFileFn<M>, HoveredFileFnAny);
fn_any!(HoveredFileCancelledFn<M>, HoveredFileCancelledFnAny);
fn_any!(DroppedFileFn<M>, DroppedFileFnAny);
fn_any!(FocusedFn<M>, FocusedFnAny);
fn_any!(UnfocusedFn<M>, UnfocusedFnAny);
fn_any!(ClosedFn<M>, ClosedFnAny);

/// A nannou window.
///
/// The `Window` acts as a wrapper around the `winit::Window` and `vulkano::Surface` types and
/// manages the associated swapchain, providing a more nannou-friendly API.
#[derive(Debug)]
pub struct Window {
    pub(crate) queue: Arc<vk::Queue>,
    pub(crate) surface: Arc<Surface>,
    msaa_samples: u32,
    pub(crate) swapchain: Arc<WindowSwapchain>,
    // Data for rendering a `Frame`'s intermediary image to a swapchain image.
    pub(crate) frame_render_data: Option<frame::RenderData>,
    pub(crate) frame_count: u64,
    pub(crate) user_functions: UserFunctions,
    // If the user specified one of the following parameters, use these when recreating the
    // swapchain rather than our heuristics.
    pub(crate) user_specified_present_mode: Option<vk::swapchain::PresentMode>,
    pub(crate) user_specified_image_count: Option<u32>,
}

/// The surface type associated with a winit window.
pub type Surface = vk::swapchain::Surface<winit::Window>;

/// The swapchain type associated with a winit window surface.
pub type Swapchain = vk::swapchain::Swapchain<winit::Window>;

/// The vulkan image type associated with a winit window surface.
pub type SwapchainImage = vk::image::swapchain::SwapchainImage<winit::Window>;

/// The future representing the moment that the GPU will have access to the swapchain image.
pub type SwapchainAcquireFuture = vk::swapchain::SwapchainAcquireFuture<winit::Window>;

/// A swapchain and its images associated with a single window.
pub(crate) struct WindowSwapchain {
    // Tracks whether or not the swapchain needs recreation due to resizing, etc.
    pub(crate) needs_recreation: AtomicBool,
    // The index of the frame at which this swapchain was first presented.
    //
    // This is necessary for allowing the user to determine whether or not they need to recreate
    // framebuffers in the case that the swapchain has recently been recreated.
    pub(crate) frame_created: u64,
    pub(crate) swapchain: Arc<Swapchain>,
    pub(crate) images: Vec<Arc<SwapchainImage>>,
    // In the application loop we are going to submit commands to the GPU. Submitting a command
    // produces an object that implements the `GpuFuture` trait, which holds the resources for as
    // long as they are in use by the GPU.
    //
    // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
    // that, we store the submission of the previous frame here.
    pub(crate) previous_frame_end: Mutex<Option<vk::FenceSignalFuture<Box<dyn vk::GpuFuture>>>>,
}

/// Swapchain building parameters for which Nannou will provide a default if unspecified.
///
/// See the builder methods for more details on each parameter.
///
/// Valid parameters can be determined prior to building by checking the result of
/// [vk::swapchain::Surface::capabilities](https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html#method.capabilities).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwapchainBuilder {
    pub format: Option<vk::Format>,
    pub color_space: Option<vk::swapchain::ColorSpace>,
    pub layers: Option<u32>,
    pub present_mode: Option<vk::swapchain::PresentMode>,
    pub composite_alpha: Option<vk::swapchain::CompositeAlpha>,
    pub clipped: Option<bool>,
    pub image_count: Option<u32>,
    pub surface_transform: Option<vk::swapchain::SurfaceTransform>,
}

/// A helper type for managing framebuffers associated with a window's swapchain images.
///
/// Creating the swapchain image framebuffers manually and maintaining them throughout the duration
/// of a program can be a tedious task that requires a lot of boilerplate code. This type
/// simplifies the process with a single `update` method that creates or recreates the framebuffers
/// if any of the following conditions are met:
/// - The given render pass is different to that which was used to create the existing
///   framebuffers.
/// - There are less framebuffers than the given frame's swapchain image index indicates are
///   required.
/// - The `frame.swapchain_image_is_new()` method indicates that the swapchain or its images have
///   recently been recreated and the framebuffers should be recreated accordingly.
#[derive(Default)]
pub struct SwapchainFramebuffers {
    framebuffers: Vec<Arc<dyn vk::FramebufferAbstract + Send + Sync>>,
}

pub type SwapchainFramebufferBuilder<A> =
    vk::FramebufferBuilder<Arc<dyn vk::RenderPassAbstract + Send + Sync>, A>;
pub type FramebufferBuildResult<A> =
    Result<SwapchainFramebufferBuilder<A>, vk::FramebufferCreationError>;

impl SwapchainFramebuffers {
    /// Ensure the framebuffers are up to date with the render pass and frame's swapchain image.
    pub fn update<F, A>(
        &mut self,
        frame: &RawFrame,
        render_pass: Arc<dyn vk::RenderPassAbstract + Send + Sync>,
        builder: F,
    ) -> Result<(), vk::FramebufferCreationError>
    where
        F: Fn(SwapchainFramebufferBuilder<()>, Arc<SwapchainImage>) -> FramebufferBuildResult<A>,
        A: 'static + vk::AttachmentsList + Send + Sync,
    {
        let mut just_created = false;
        while frame.swapchain_image_index() >= self.framebuffers.len() {
            let builder = builder(
                vk::Framebuffer::start(render_pass.clone()),
                frame.swapchain_image().clone(),
            )?;
            let fb = builder.build()?;
            self.framebuffers.push(Arc::new(fb));
            just_created = true;
        }

        // If the dimensions for the current framebuffer do not match, recreate it.
        let old_rp =
            vk::RenderPassAbstract::inner(&self.framebuffers[frame.swapchain_image_index()])
                .internal_object();
        let new_rp = render_pass.inner().internal_object();
        if !just_created && (frame.swapchain_image_is_new() || old_rp != new_rp) {
            let fb = &mut self.framebuffers[frame.swapchain_image_index()];
            let builder = builder(
                vk::Framebuffer::start(render_pass.clone()),
                frame.swapchain_image().clone(),
            )?;
            let new_fb = builder.build()?;
            *fb = Arc::new(new_fb);
        }
        Ok(())
    }
}

impl ops::Deref for SwapchainFramebuffers {
    type Target = [Arc<dyn vk::FramebufferAbstract + Send + Sync>];
    fn deref(&self) -> &Self::Target {
        &self.framebuffers
    }
}

/// The errors that might occur while constructing a `Window`.
#[derive(Debug)]
pub enum BuildError {
    SurfaceCreation(vk::win::CreationError),
    DeviceCreation(vk::DeviceCreationError),
    SwapchainCreation(vk::SwapchainCreationError),
    SwapchainCapabilities(vk::swapchain::CapabilitiesError),
    RenderDataCreation(frame::RenderDataCreationError),
    SurfaceDoesNotSupportCompositeAlphaOpaque,
}

impl SwapchainBuilder {
    pub const DEFAULT_CLIPPED: bool = true;
    pub const DEFAULT_COLOR_SPACE: vk::swapchain::ColorSpace =
        vk::swapchain::ColorSpace::SrgbNonLinear;
    pub const DEFAULT_COMPOSITE_ALPHA: vk::swapchain::CompositeAlpha =
        vk::swapchain::CompositeAlpha::Opaque;
    pub const DEFAULT_LAYERS: u32 = 1;
    pub const DEFAULT_SURFACE_TRANSFORM: vk::swapchain::SurfaceTransform =
        vk::swapchain::SurfaceTransform::Identity;

    /// A new empty **SwapchainBuilder** with all parameters set to `None`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a **SwapchainBuilder** from an existing swapchain.
    ///
    /// The resulting swapchain parameters will match that of the given `Swapchain`.
    ///
    /// Note that `sharing_mode` will be `None` regardless of how the given `Swapchain` was built,
    /// as there is no way to determine this via the vulkano swapchain API.
    pub fn from_swapchain(swapchain: &Swapchain) -> Self {
        SwapchainBuilder::new()
            .format(swapchain.format())
            .image_count(swapchain.num_images())
            .layers(swapchain.layers())
            .surface_transform(swapchain.transform())
            .composite_alpha(swapchain.composite_alpha())
            .present_mode(swapchain.present_mode())
            .clipped(swapchain.clipped())
    }

    /// Specify the pixel format for the swapchain.
    ///
    /// By default, nannou attempts to use the first format valid for the `SrgbNonLinear` color
    /// space.
    ///
    /// See the [vulkano docs](https://docs.rs/vulkano/latest/vulkano/format/enum.Format.html).
    pub fn format(mut self, format: vk::Format) -> Self {
        self.format = Some(format);
        self
    }

    /// If `format` is `None`, will attempt to find the first available `Format` that supports this
    /// `ColorSpace`.
    ///
    /// If `format` is `Some`, this parameter is ignored.
    ///
    /// By default, nannou attempts to use the first format valid for the `SrgbNonLinear` color
    /// space.
    ///
    /// See the [vulkano docs](https://docs.rs/vulkano/latest/vulkano/swapchain/enum.ColorSpace.html).
    pub fn color_space(mut self, color_space: vk::swapchain::ColorSpace) -> Self {
        self.color_space = Some(color_space);
        self
    }

    /// How the alpha values of the pixels of the window are treated.
    ///
    /// By default, nannou uses `CompositeAlpha::Opaque`.
    ///
    /// See the [vulkano docs](https://docs.rs/vulkano/latest/vulkano/swapchain/enum.CompositeAlpha.html).
    pub fn composite_alpha(mut self, composite_alpha: vk::swapchain::CompositeAlpha) -> Self {
        self.composite_alpha = Some(composite_alpha);
        self
    }

    /// The way in which swapchain images are presented to the display.
    ///
    /// By default, nannou will attempt to select the ideal present mode depending on the current
    /// app `LoopMode`. If the current loop mode is `Wait` or `Rate`, nannou will attempt to use
    /// the `Mailbox` present mode with an `image_count` of `3`. If the current loop mode is
    /// `RefreshSync`, nannou will use the `Fifo` present m ode with an `image_count` of `2`.
    ///
    /// See the [vulkano docs](https://docs.rs/vulkano/latest/vulkano/swapchain/enum.PresentMode.html).
    pub fn present_mode(mut self, present_mode: vk::swapchain::PresentMode) -> Self {
        self.present_mode = Some(present_mode);
        self
    }

    /// The number of images used by the swapchain.
    ///
    /// By default, nannou will attempt to select the ideal image count depending on the current
    /// app `LoopMode`. If the current loop mode is `Wait` or `Rate`, nannou will attempt to use
    /// the `Mailbox` present mode with an `image_count` of `3`. If the current loop mode is
    /// `RefreshSync`, nannou will use the `Fifo` present m ode with an `image_count` of `2`.
    pub fn image_count(mut self, image_count: u32) -> Self {
        self.image_count = Some(image_count);
        self
    }

    /// Whether the implementation is allowed to discard rendering operations that affect regions
    /// of the surface which aren't visible.
    ///
    /// This is important to take into account if your fragment shader has side-effects or if you
    /// want to read back the content of the image afterwards.
    pub fn clipped(mut self, clipped: bool) -> Self {
        self.clipped = Some(clipped);
        self
    }

    /// A transformation to apply to the image before showing it on the screen.
    ///
    /// See the [vulkano docs](https://docs.rs/vulkano/latest/vulkano/swapchain/enum.SurfaceTransform.html).
    pub fn surface_transform(mut self, surface_transform: vk::swapchain::SurfaceTransform) -> Self {
        self.surface_transform = Some(surface_transform);
        self
    }

    pub fn layers(mut self, layers: u32) -> Self {
        self.layers = Some(layers);
        self
    }

    /// Build the swapchain.
    ///
    /// `fallback_dimensions` are dimensions to use in the case that the surface capabilities
    /// `current_extent` field is `None`, which may happen if a surface's size is determined by the
    /// swapchain's size.
    pub(crate) fn build<S>(
        self,
        device: Arc<vk::Device>,
        surface: Arc<Surface>,
        sharing_mode: S,
        loop_mode: &LoopMode,
        fallback_dimensions: Option<[u32; 2]>,
        old_swapchain: Option<&Arc<Swapchain>>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<SwapchainImage>>), vk::SwapchainCreationError>
    where
        S: Into<vk::sync::SharingMode>,
    {
        let capabilities = surface
            .capabilities(device.physical_device())
            .expect("failed to retrieve surface capabilities");

        let dimensions = capabilities
            .current_extent
            .or(fallback_dimensions)
            .unwrap_or([
                DEFAULT_DIMENSIONS.width as _,
                DEFAULT_DIMENSIONS.height as _,
            ]);

        // Retrieve the format.
        let format = match self.format {
            Some(fmt) => fmt,
            None => {
                let color_space = self.color_space.unwrap_or(Self::DEFAULT_COLOR_SPACE);

                // First, try to pick an Srgb format.
                capabilities
                    .supported_formats
                    .iter()
                    .filter(|&&(fmt, cs)| vk::format_is_srgb(fmt) && cs == color_space)
                    .next()
                    .or_else(|| {
                        // Otherwise just try and math the color space.
                        capabilities
                            .supported_formats
                            .iter()
                            .filter(|&&(_, cs)| cs == color_space)
                            .next()
                    })
                    .map(|&(fmt, _cs)| fmt)
                    .ok_or(vk::SwapchainCreationError::UnsupportedFormat)?
            }
        };

        // Determine the optimal present mode and image count based on the specified parameters and
        // the current loop mode.
        let (present_mode, image_count) = preferred_present_mode_and_image_count(
            &loop_mode,
            capabilities.min_image_count,
            self.present_mode,
            self.image_count,
            &capabilities.present_modes,
        );

        // Attempt to retrieve the desired composite alpha.
        let composite_alpha = match self.composite_alpha {
            Some(alpha) => alpha,
            None => match capabilities.supported_composite_alpha.opaque {
                true => Self::DEFAULT_COMPOSITE_ALPHA,
                false => return Err(vk::SwapchainCreationError::UnsupportedCompositeAlpha),
            },
        };

        let layers = self.layers.unwrap_or(Self::DEFAULT_LAYERS);
        let clipped = self.clipped.unwrap_or(Self::DEFAULT_CLIPPED);
        let surface_transform = self
            .surface_transform
            .unwrap_or(Self::DEFAULT_SURFACE_TRANSFORM);

        Swapchain::new(
            device,
            surface,
            image_count,
            format,
            dimensions,
            layers,
            capabilities.supported_usage_flags,
            sharing_mode,
            surface_transform,
            composite_alpha,
            present_mode,
            clipped,
            old_swapchain,
        )
    }
}

/// Determine the optimal present mode and image count for the given loop mode.
///
/// If a specific present mode or image count is desired, they may be optionally specified.
pub fn preferred_present_mode_and_image_count(
    loop_mode: &LoopMode,
    min_image_count: u32,
    present_mode: Option<vk::swapchain::PresentMode>,
    image_count: Option<u32>,
    supported_present_modes: &vk::swapchain::SupportedPresentModes,
) -> (vk::swapchain::PresentMode, u32) {
    match (present_mode, image_count) {
        (Some(pm), Some(ic)) => (pm, ic),
        (None, _) => match *loop_mode {
            LoopMode::RefreshSync { .. } => {
                let image_count = image_count.unwrap_or_else(|| cmp::max(min_image_count, 2));
                (vk::swapchain::PresentMode::Fifo, image_count)
            }
            LoopMode::Wait { .. } | LoopMode::Rate { .. } | LoopMode::NTimes { .. } => {
                if supported_present_modes.mailbox {
                    let image_count = image_count.unwrap_or_else(|| cmp::max(min_image_count, 3));
                    (vk::swapchain::PresentMode::Mailbox, image_count)
                } else {
                    let image_count = image_count.unwrap_or_else(|| cmp::max(min_image_count, 2));
                    (vk::swapchain::PresentMode::Fifo, image_count)
                }
            }
        },
        (Some(present_mode), None) => {
            let image_count = match present_mode {
                vk::swapchain::PresentMode::Immediate => min_image_count,
                vk::swapchain::PresentMode::Mailbox => cmp::max(min_image_count, 3),
                vk::swapchain::PresentMode::Fifo => cmp::max(min_image_count, 2),
                vk::swapchain::PresentMode::Relaxed => cmp::max(min_image_count, 2),
            };
            (present_mode, image_count)
        }
    }
}

impl<'app> Builder<'app> {
    /// Begin building a new window.
    pub fn new(app: &'app App) -> Self {
        Builder {
            app,
            vk_physical_device: None,
            vk_device_extensions: None,
            vk_device_queue: None,
            window: winit::WindowBuilder::new(),
            title_was_set: false,
            swapchain_builder: Default::default(),
            user_functions: Default::default(),
            msaa_samples: None,
        }
    }

    /// Build the window with some custom window parameters.
    pub fn window(mut self, window: winit::WindowBuilder) -> Self {
        self.window = window;
        self
    }

    /// The physical device to associate with the window surface's swapchain.
    pub fn vk_physical_device(mut self, device: vk::PhysicalDevice<'app>) -> Self {
        self.vk_physical_device = Some(device);
        self
    }

    /// Specify a set of required extensions.
    ///
    /// The device associated with the window's swapchain *must* always have the `khr_swapchain`
    /// feature enabled, so it will be implicitly enabled whether or not it is specified in this
    /// given set of extensions.
    pub fn vk_device_extensions(mut self, extensions: vk::DeviceExtensions) -> Self {
        self.vk_device_extensions = Some(extensions);
        self
    }

    /// Specify the vulkan device queue for this window.
    ///
    /// This queue is used as the `SharingMode` for the swapchain, and is also used for
    /// constructing the `Frame`'s intermediary image.
    ///
    /// Once the window is built, this queue can be accessed via the `window.swapchain_queue()`
    /// method.
    ///
    /// Note: If this builder method is called, previous calls to `vk_physical_device` and
    /// `vk_device_extensions` will be ignored as specifying the queue for the sharing mode
    /// implies which logical device is desired.
    pub fn vk_device_queue(mut self, queue: Arc<vk::Queue>) -> Self {
        self.vk_device_queue = Some(queue);
        self
    }

    /// Specify a set of parameters for building the window surface swapchain.
    pub fn swapchain_builder(mut self, swapchain_builder: SwapchainBuilder) -> Self {
        self.swapchain_builder = swapchain_builder;
        self
    }

    /// Specify the number of samples per pixel for the multisample anti-aliasing render pass.
    ///
    /// If `msaa_samples` is unspecified, the first default value that nannou will attempt to use
    /// can be found via the `Frame::DEFAULT_MSAA_SAMPLES` constant. If however this value is not
    /// supported by the window's swapchain, nannou will fallback to the next smaller power of 2
    /// that is supported. If MSAA is not supported at all, then the default will be 1.
    ///
    /// **Note:** This parameter has no meaning if the window uses a **raw_view** function for
    /// rendering graphics to the window rather than a **view** function. This is because the
    /// **raw_view** function provides a **RawFrame** with direct access to the swapchain image
    /// itself and thus must manage their own MSAA pass.
    ///
    /// On the other hand, the `view` function provides the `Frame` type which allows the user to
    /// render to a multisampled intermediary image allowing Nannou to take care of resolving the
    /// multisampled image to the swapchain image. In order to avoid confusion, The `Window::build`
    /// method will `panic!` if the user tries to specify `msaa_samples` as well as a `raw_view`
    /// method.
    ///
    /// *TODO: Perhaps it would be worth adding two separate methods for specifying msaa samples.
    /// One for forcing a certain number of samples and returning an error otherwise, and another
    /// for attempting to use the given number of samples but falling back to a supported value in
    /// the case that the specified number is not supported.*
    pub fn msaa_samples(mut self, msaa_samples: u32) -> Self {
        self.msaa_samples = Some(msaa_samples);
        self
    }

    /// Provide a simple function for drawing to the window.
    ///
    /// This is similar to `view` but does not provide access to user data via a Model type. This
    /// is useful for sketches where you don't require tracking any state.
    pub fn sketch(mut self, sketch_fn: SketchFn) -> Self {
        self.user_functions.view = Some(View::Sketch(sketch_fn));
        self
    }

    /// The **view** function that the app will call to allow you to present your Model to the
    /// surface of the window on your display.
    pub fn view<M>(mut self, view_fn: ViewFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.view = Some(View::WithModel(ViewFnAny::from_fn_ptr(view_fn)));
        self
    }

    /// The **view** function that the app will call to allow you to present your Model to the
    /// surface of the window on your display.
    ///
    /// Unlike the **ViewFn**, the **RawViewFn** provides a **RawFrame** that is designed for
    /// drawing directly to a window's swapchain images, rather than to a convenient intermediary
    /// image.
    pub fn raw_view<M>(mut self, raw_view_fn: RawViewFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.view = Some(View::WithModelRaw(RawViewFnAny::from_fn_ptr(raw_view_fn)));
        self
    }

    /// A function for updating your model on `WindowEvent`s associated with this window.
    ///
    /// These include events such as key presses, mouse movement, clicks, resizing, etc.
    ///
    /// ## Event Function Call Order
    ///
    /// In nannou, if multiple functions require being called for a single kind of event, the more
    /// general event function will always be called before the more specific event function.
    ///
    /// If an `event` function was also submitted to the `App`, that function will always be called
    /// immediately before window-specific event functions. Similarly, if a function associated
    /// with a more specific event type (e.g. `key_pressed`) was given, that function will be
    /// called *after* this function will be called.
    ///
    /// ## Specific Events Variants
    ///
    /// Note that if you only care about a certain kind of event, you can submit a function that
    /// only gets called for that specific event instead. For example, if you only care about key
    /// presses, you may wish to use the `key_pressed` method instead.
    pub fn event<M>(mut self, event_fn: EventFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.event = Some(EventFnAny::from_fn_ptr(event_fn));
        self
    }

    /// The same as the `event` method, but allows for processing raw `winit::WindowEvent`s rather
    /// than Nannou's simplified `event::WindowEvent`s.
    ///
    /// ## Event Function Call Order
    ///
    /// If both `raw_event` and `event` functions have been provided, the given `raw_event`
    /// function will always be called immediately before the given `event` function.
    pub fn raw_event<M>(mut self, raw_event_fn: RawEventFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.raw_event = Some(RawEventFnAny::from_fn_ptr(raw_event_fn));
        self
    }

    /// A function for processing key press events associated with this window.
    pub fn key_pressed<M>(mut self, f: KeyPressedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.key_pressed = Some(KeyPressedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing key release events associated with this window.
    pub fn key_released<M>(mut self, f: KeyReleasedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.key_released = Some(KeyReleasedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing mouse moved events associated with this window.
    pub fn mouse_moved<M>(mut self, f: MouseMovedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.mouse_moved = Some(MouseMovedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing mouse pressed events associated with this window.
    pub fn mouse_pressed<M>(mut self, f: MousePressedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.mouse_pressed = Some(MousePressedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing mouse released events associated with this window.
    pub fn mouse_released<M>(mut self, f: MouseReleasedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.mouse_released = Some(MouseReleasedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing mouse wheel events associated with this window.
    pub fn mouse_wheel<M>(mut self, f: MouseWheelFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.mouse_wheel = Some(MouseWheelFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing mouse entered events associated with this window.
    pub fn mouse_entered<M>(mut self, f: MouseEnteredFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.mouse_entered = Some(MouseEnteredFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing mouse exited events associated with this window.
    pub fn mouse_exited<M>(mut self, f: MouseExitedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.mouse_exited = Some(MouseExitedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing touch events associated with this window.
    pub fn touch<M>(mut self, f: TouchFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.touch = Some(TouchFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing touchpad pressure events associated with this window.
    pub fn touchpad_pressure<M>(mut self, f: TouchpadPressureFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.touchpad_pressure = Some(TouchpadPressureFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing window moved events associated with this window.
    pub fn moved<M>(mut self, f: MovedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.moved = Some(MovedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing window resized events associated with this window.
    pub fn resized<M>(mut self, f: ResizedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.resized = Some(ResizedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing hovered file events associated with this window.
    pub fn hovered_file<M>(mut self, f: HoveredFileFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.hovered_file = Some(HoveredFileFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing hovered file cancelled events associated with this window.
    pub fn hovered_file_cancelled<M>(mut self, f: HoveredFileCancelledFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.hovered_file_cancelled =
            Some(HoveredFileCancelledFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing dropped file events associated with this window.
    pub fn dropped_file<M>(mut self, f: DroppedFileFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.dropped_file = Some(DroppedFileFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing the focused event associated with this window.
    pub fn focused<M>(mut self, f: FocusedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.focused = Some(FocusedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing the unfocused event associated with this window.
    pub fn unfocused<M>(mut self, f: UnfocusedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.unfocused = Some(UnfocusedFnAny::from_fn_ptr(f));
        self
    }

    /// A function for processing the window closed event associated with this window.
    pub fn closed<M>(mut self, f: ClosedFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.closed = Some(ClosedFnAny::from_fn_ptr(f));
        self
    }

    /// Builds the window, inserts it into the `App`'s display map and returns the unique ID.
    pub fn build(self) -> Result<Id, BuildError> {
        let Builder {
            app,
            vk_physical_device,
            vk_device_extensions,
            vk_device_queue,
            mut window,
            title_was_set,
            swapchain_builder,
            user_functions,
            msaa_samples,
        } = self;

        // If the title was not set, default to the "nannou - <exe_name>".
        if !title_was_set {
            if let Ok(exe_path) = env::current_exe() {
                if let Some(os_str) = exe_path.file_stem() {
                    if let Some(exe_name) = os_str.to_str() {
                        let title = format!("nannou - {}", exe_name);
                        window = window.with_title(title);
                    }
                }
            }
        }

        // Retrieve dimensions to use as a fallback in case vulkano swapchain capabilities
        // `current_extent` is `None`. This happens when the window size is determined by the size
        // of the swapchain.
        let initial_swapchain_dimensions = window
            .window
            .dimensions
            .or_else(|| {
                window
                    .window
                    .fullscreen
                    .as_ref()
                    .map(|monitor| monitor.get_dimensions().to_logical(1.0))
            })
            .unwrap_or_else(|| {
                let mut dim = DEFAULT_DIMENSIONS;
                if let Some(min) = window.window.min_dimensions {
                    dim.width = dim.width.max(min.width);
                    dim.height = dim.height.max(min.height);
                }
                if let Some(max) = window.window.max_dimensions {
                    dim.width = dim.width.min(max.width);
                    dim.height = dim.height.min(max.height);
                }
                dim
            });

        // Use the `initial_swapchain_dimensions` as the default dimensions for the window if none
        // were specified.
        if window.window.dimensions.is_none() && window.window.fullscreen.is_none() {
            window.window.dimensions = Some(initial_swapchain_dimensions);
        }

        // Build the vulkan surface.
        let surface = window.build_vk_surface(&app.events_loop, app.vk_instance.clone())?;

        // The logical device queue to use as the swapchain sharing mode.
        // This queue will also be used for constructing the `Frame`'s intermediary image.
        let queue = match vk_device_queue {
            Some(queue) => queue,
            None => {
                // Retrieve the physical, vulkan-supported device to use.
                let physical_device = vk_physical_device
                    .or_else(|| app.default_vk_physical_device())
                    .unwrap_or_else(|| unimplemented!());

                // Select the queue family to use. Default to the first graphics-supporting queue.
                let queue_family = physical_device
                    .queue_families()
                    .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                    .unwrap_or_else(|| unimplemented!("couldn't find a graphical queue family"));

                // We only have one queue, so give an arbitrary priority.
                let queue_priority = 0.5;

                // The required device extensions.
                let mut device_ext =
                    vk_device_extensions.unwrap_or_else(vk::DeviceExtensions::none);
                device_ext.khr_swapchain = true;

                // Enable all supported device features.
                let features = physical_device.supported_features();

                // Construct the logical device and queues.
                let (_device, mut queues) = vk::Device::new(
                    physical_device,
                    features,
                    &device_ext,
                    [(queue_family, queue_priority)].iter().cloned(),
                )?;

                // Since it is possible to request multiple queues, the queues variable returned by
                // the function is in fact an iterator. In this case this iterator contains just
                // one element, so let's extract it.
                let queue = queues.next().expect("expected a single device queue");
                queue
            }
        };

        let user_specified_present_mode = swapchain_builder.present_mode;
        let user_specified_image_count = swapchain_builder.image_count;

        // Build the swapchain used for displaying the window contents.
        let (swapchain, images) = {
            // Set the dimensions of the swapchain to that of the surface.
            let fallback_dimensions = [
                initial_swapchain_dimensions.width as _,
                initial_swapchain_dimensions.height as _,
            ];

            swapchain_builder.build(
                queue.device().clone(),
                surface.clone(),
                &queue,
                &app.loop_mode(),
                Some(fallback_dimensions),
                None,
            )?
        };

        // If we're using an intermediary image for rendering frames to swapchain images, create
        // the necessary render data.
        let (frame_render_data, msaa_samples) = match user_functions.view {
            Some(View::WithModel(_)) | Some(View::Sketch(_)) | None => {
                let target_msaa_samples = msaa_samples.unwrap_or(Frame::DEFAULT_MSAA_SAMPLES);
                let physical_device = queue.device().physical_device();
                let msaa_samples = vk::msaa_samples_limited(&physical_device, target_msaa_samples);
                let render_data = frame::RenderData::new(
                    queue.device().clone(),
                    swapchain.dimensions(),
                    msaa_samples,
                    swapchain.format(),
                )?;
                (Some(render_data), msaa_samples)
            }
            Some(View::WithModelRaw(_)) => (None, 1),
        };

        let window_id = surface.window().id();
        let needs_recreation = AtomicBool::new(false);
        let previous_frame_end = Mutex::new(None);
        let frame_count = 0;
        let swapchain = Arc::new(WindowSwapchain {
            needs_recreation,
            frame_created: frame_count,
            swapchain,
            images,
            previous_frame_end,
        });
        let window = Window {
            queue,
            surface,
            msaa_samples,
            swapchain,
            frame_render_data,
            frame_count,
            user_functions,
            user_specified_present_mode,
            user_specified_image_count,
        };
        app.windows.borrow_mut().insert(window_id, window);

        // If this is the first window, set it as the app's "focused" window.
        if app.windows.borrow().len() == 1 {
            *app.focused_window.borrow_mut() = Some(window_id);
        }

        Ok(window_id)
    }

    fn map_window<F>(self, map: F) -> Self
    where
        F: FnOnce(winit::WindowBuilder) -> winit::WindowBuilder,
    {
        let Builder {
            app,
            vk_physical_device,
            vk_device_extensions,
            vk_device_queue,
            window,
            title_was_set,
            swapchain_builder,
            user_functions,
            msaa_samples,
        } = self;
        let window = map(window);
        Builder {
            app,
            vk_physical_device,
            vk_device_extensions,
            vk_device_queue,
            window,
            title_was_set,
            swapchain_builder,
            user_functions,
            msaa_samples,
        }
    }

    // Window builder methods.

    /// Requests the window to be specific dimensions pixels.
    pub fn with_dimensions(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_dimensions((width, height).into()))
    }

    /// Set the minimum dimensions in pixels for the window.
    pub fn with_min_dimensions(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_min_dimensions((width, height).into()))
    }

    /// Set the maximum dimensions in pixels for the window.
    pub fn with_max_dimensions(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_max_dimensions((width, height).into()))
    }

    /// Requests a specific title for the window.
    pub fn with_title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title_was_set = true;
        self.map_window(|w| w.with_title(title))
    }

    /// Sets the window fullscreen state.
    ///
    /// None means a normal window, Some(MonitorId) means a fullscreen window on that specific
    /// monitor.
    pub fn with_fullscreen(self, monitor: Option<MonitorId>) -> Self {
        self.map_window(|w| w.with_fullscreen(monitor))
    }

    /// Requests maximized mode.
    pub fn with_maximized(self, maximized: bool) -> Self {
        self.map_window(|w| w.with_maximized(maximized))
    }

    /// Sets whether the window will be initially hidden or visible.
    pub fn with_visibility(self, visible: bool) -> Self {
        self.map_window(|w| w.with_visibility(visible))
    }

    /// Sets whether the background of the window should be transparent.
    pub fn with_transparency(self, transparent: bool) -> Self {
        self.map_window(|w| w.with_transparency(transparent))
    }

    /// Sets whether the window should have a border, a title bar, etc.
    pub fn with_decorations(self, decorations: bool) -> Self {
        self.map_window(|w| w.with_decorations(decorations))
    }

    /// Enables multitouch.
    pub fn with_multitouch(self) -> Self {
        self.map_window(|w| w.with_multitouch())
    }
}

impl Window {
    const NO_LONGER_EXISTS: &'static str = "the window no longer exists";

    // `winit::Window` methods.

    /// Modifies the title of the window.
    ///
    /// This is a no-op if the window has already been closed.
    pub fn set_title(&self, title: &str) {
        self.surface.window().set_title(title);
    }

    /// Shows the window if it was hidden.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn show(&self) {
        self.surface.window().show()
    }

    /// Hides the window if it was visible.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn hide(&self) {
        self.surface.window().hide()
    }

    /// The position of the top-left hand corner of the window relative to the top-left hand corner
    /// of the desktop.
    ///
    /// Note that the top-left hand corner of the desktop is not necessarily the same as the
    /// screen. If the user uses a desktop with multiple monitors, the top-left hand corner of the
    /// desktop is the top-left hand corner of the monitor at the top-left of the desktop.
    ///
    /// The coordinates can be negative if the top-left hand corner of the window is outside of the
    /// visible screen region.
    pub fn position(&self) -> (i32, i32) {
        self.surface
            .window()
            .get_position()
            .expect(Self::NO_LONGER_EXISTS)
            .into()
    }

    /// Modifies the position of the window.
    ///
    /// See `get_position` for more information about the returned coordinates.
    pub fn set_position(&self, x: i32, y: i32) {
        self.surface.window().set_position((x, y).into())
    }

    /// The size in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These
    /// are the dimensions of the frame buffer, and the dimensions that you should use when you
    /// call glViewport.
    pub fn inner_size_pixels(&self) -> (u32, u32) {
        self.surface
            .window()
            .get_inner_size()
            .map(|logical_px| {
                let hidpi_factor = self.surface.window().get_hidpi_factor();
                logical_px.to_physical(hidpi_factor)
            })
            .expect(Self::NO_LONGER_EXISTS)
            .into()
    }

    /// The size in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. To get
    /// the dimensions of the frame buffer when calling `glViewport`, multiply with hidpi factor.
    ///
    /// This is the same as dividing the result  of `inner_size_pixels()` by `hidpi_factor()`.
    pub fn inner_size_points(&self) -> (geom::scalar::Default, geom::scalar::Default) {
        let size = self
            .surface
            .window()
            .get_inner_size()
            .expect(Self::NO_LONGER_EXISTS);
        let (w, h): (f64, f64) = size.into();
        (w as _, h as _)
    }

    /// The size of the window in pixels.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    /// `inner_size_pixels` instead.
    pub fn outer_size_pixels(&self) -> (u32, u32) {
        self.surface
            .window()
            .get_outer_size()
            .map(|logical_px| {
                let hidpi_factor = self.surface.window().get_hidpi_factor();
                logical_px.to_physical(hidpi_factor)
            })
            .expect(Self::NO_LONGER_EXISTS)
            .into()
    }

    /// The size of the window in points.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    /// `inner_size_points` instead.
    ///
    /// This is the same as dividing the result  of `outer_size_pixels()` by `hidpi_factor()`.
    pub fn outer_size_points(&self) -> (f32, f32) {
        let size = self
            .surface
            .window()
            .get_outer_size()
            .expect(Self::NO_LONGER_EXISTS);
        let (w, h): (f64, f64) = size.into();
        (w as _, h as _)
    }

    /// Modifies the inner size of the window.
    ///
    /// See the `inner_size` methods for more informations about the values.
    pub fn set_inner_size_pixels(&self, width: u32, height: u32) {
        self.surface.window().set_inner_size((width, height).into())
    }

    /// Modifies the inner size of the window using point values.
    ///
    /// Internally, the given width and height are multiplied by the `hidpi_factor` to get the
    /// values in pixels before calling `set_inner_size_pixels` internally.
    pub fn set_inner_size_points(&self, width: f32, height: f32) {
        let hidpi_factor = self.hidpi_factor();
        let w_px = (width * hidpi_factor) as _;
        let h_px = (height * hidpi_factor) as _;
        self.set_inner_size_pixels(w_px, h_px);
    }

    /// The ratio between the backing framebuffer resolution and the window size in screen pixels.
    ///
    /// This is typically `1.0` for a normal display, `2.0` for a retina display and higher on more
    /// modern displays.
    pub fn hidpi_factor(&self) -> geom::scalar::Default {
        self.surface.window().get_hidpi_factor() as _
    }

    /// Changes the position of the cursor in window coordinates.
    pub fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), String> {
        self.surface.window().set_cursor_position((x, y).into())
    }

    /// Modifies the mouse cursor of the window.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn set_cursor(&self, state: MouseCursor) {
        self.surface.window().set_cursor(state);
    }

    /// Grabs the cursor, preventing it from leaving the window.
    ///
    /// ## Platform-specific
    ///
    /// On macOS, this presently merely locks the cursor in a fixed location, which looks visually
    /// awkward.
    ///
    /// This has no effect on Android or iOS.
    pub fn grab_cursor(&self, grab: bool) -> Result<(), String> {
        self.surface.window().grab_cursor(grab)
    }

    /// Hides the cursor, making it invisible but still usable.
    ///
    /// ## Platform-specific
    ///
    /// On Windows and X11, the cursor is only hidden within the confines of the window.
    ///
    /// On macOS, the cursor is hidden as long as the window has input focus, even if the cursor is
    /// outside of the window.
    ///
    /// This has no effect on Android or iOS.
    pub fn hide_cursor(&self, hide: bool) {
        self.surface.window().hide_cursor(hide)
    }

    /// Sets the window to maximized or back.
    pub fn set_maximized(&self, maximized: bool) {
        self.surface.window().set_maximized(maximized)
    }

    /// Set the window to fullscreen on the monitor associated with the given `Id`.
    ///
    /// Call this method again with `None` to revert back from fullscreen.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn set_fullscreen(&self, monitor: Option<MonitorId>) {
        self.surface.window().set_fullscreen(monitor)
    }

    /// The current monitor that the window is on or the primary monitor if nothing matches.
    pub fn current_monitor(&self) -> MonitorId {
        self.surface.window().get_current_monitor()
    }

    /// A unique identifier associated with this window.
    pub fn id(&self) -> Id {
        self.surface.window().id()
    }

    // Access to vulkano API.

    /// Returns a reference to the window's Vulkan swapchain surface.
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// The swapchain associated with this window's vulkan surface.
    pub fn swapchain(&self) -> &Swapchain {
        &self.swapchain.swapchain
    }

    /// The vulkan logical device on which the window's swapchain is running.
    ///
    /// This is shorthand for `DeviceOwned::device(window.swapchain())`.
    pub fn swapchain_device(&self) -> &Arc<vk::Device> {
        vk::DeviceOwned::device(self.swapchain())
    }

    /// The vulkan graphics queue on which the window swapchain work is run.
    pub fn swapchain_queue(&self) -> &Arc<vk::Queue> {
        &self.queue
    }

    /// The vulkan images associated with the window's swapchain.
    ///
    /// This method is exposed in order to allow for interop with low-level vulkano code (e.g.
    /// framebuffer creation). We recommend that you avoid storing these images as the swapchain
    /// and its images may be recreated at any moment in time.
    pub fn swapchain_images(&self) -> &[Arc<SwapchainImage>] {
        &self.swapchain.images
    }

    /// The number of samples used in the MSAA for the image associated with the `view` function's
    /// `Frame` type.
    ///
    /// **Note:** If the user specified a `raw_view` function rather than a `view` function, this
    /// value will always return `1`.
    pub fn msaa_samples(&self) -> u32 {
        self.msaa_samples
    }

    // Custom methods.

    // A utility function to simplify the recreation of a swapchain.
    pub(crate) fn replace_swapchain(
        &mut self,
        new_swapchain: Arc<Swapchain>,
        new_images: Vec<Arc<SwapchainImage>>,
    ) {
        let previous_frame_end = self
            .swapchain
            .previous_frame_end
            .lock()
            .expect("failed to lock `previous_frame_end`")
            .take();
        self.swapchain = Arc::new(WindowSwapchain {
            needs_recreation: AtomicBool::new(false),
            frame_created: self.frame_count,
            swapchain: new_swapchain,
            images: new_images,
            previous_frame_end: Mutex::new(previous_frame_end),
        });
        // TODO: Update frame_render_data?
    }

    /// Attempts to determine whether or not the window is currently fullscreen.
    ///
    /// TODO: This currently relies on comparing `outer_size_pixels` to the dimensions of the
    /// `current_monitor`, which may not be exactly accurate on some platforms or even conceptually
    /// correct in the case that a title bar is included or something. This should probably be a
    /// method upstream within the `winit` crate itself. Alternatively we could attempt to manually
    /// track whether or not the window is fullscreen ourselves, however this could get quite
    /// complicated quite quickly.
    pub fn is_fullscreen(&self) -> bool {
        let (w, h) = self.outer_size_pixels();
        let (mw, mh): (u32, u32) = self.current_monitor().get_dimensions().into();
        w == mw && h == mh
    }

    /// The number of times `view` has been called with a `Frame` for this window.
    pub fn elapsed_frames(&self) -> u64 {
        self.frame_count
    }
}

// Debug implementations for function wrappers.

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variant = match *self {
            View::WithModel(ref v) => format!("WithModel({:?})", v),
            View::WithModelRaw(ref v) => format!("WithModelRaw({:?})", v),
            View::Sketch(_) => "Sketch".to_string(),
        };
        write!(f, "View::{}", variant)
    }
}

// Deref implementations.

impl ops::Deref for WindowSwapchain {
    type Target = Arc<Swapchain>;
    fn deref(&self) -> &Self::Target {
        &self.swapchain
    }
}

impl fmt::Debug for WindowSwapchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WindowSwapchain ( swapchain: {:?}, swapchain_images: {:?} )",
            self.swapchain,
            self.images.len(),
        )
    }
}

// Error implementations.

impl StdError for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::SurfaceCreation(ref err) => err.description(),
            BuildError::DeviceCreation(ref err) => err.description(),
            BuildError::SwapchainCreation(ref err) => err.description(),
            BuildError::SwapchainCapabilities(ref err) => err.description(),
            BuildError::RenderDataCreation(ref err) => err.description(),
            BuildError::SurfaceDoesNotSupportCompositeAlphaOpaque => {
                "`CompositeAlpha::Opaque` not supported by window surface"
            }
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<vk::win::CreationError> for BuildError {
    fn from(e: vk::win::CreationError) -> Self {
        BuildError::SurfaceCreation(e)
    }
}

impl From<vk::DeviceCreationError> for BuildError {
    fn from(e: vk::DeviceCreationError) -> Self {
        BuildError::DeviceCreation(e)
    }
}

impl From<vk::swapchain::SwapchainCreationError> for BuildError {
    fn from(e: vk::swapchain::SwapchainCreationError) -> Self {
        BuildError::SwapchainCreation(e)
    }
}

impl From<vk::swapchain::CapabilitiesError> for BuildError {
    fn from(e: vk::swapchain::CapabilitiesError) -> Self {
        BuildError::SwapchainCapabilities(e)
    }
}

impl From<frame::RenderDataCreationError> for BuildError {
    fn from(e: frame::RenderDataCreationError) -> Self {
        BuildError::RenderDataCreation(e)
    }
}
