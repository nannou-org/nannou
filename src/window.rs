//! The nannou [**Window**](./struct.Window.html) API. Create a new window via `.app.new_window()`.
//! This produces a [**Builder**](./struct.Builder.html) which can be used to build a window.

use app::LoopMode;
use geom;
use std::{cmp, env, fmt, ops};
use std::error::Error as StdError;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use vulkano::device::{self, Device};
use vulkano::format::Format;
use vulkano::instance::PhysicalDevice;
use vulkano::swapchain::{ColorSpace, CompositeAlpha, PresentMode, SurfaceTransform,
                         SwapchainCreationError};
use vulkano::sync::GpuFuture;
use vulkano_win::{VkSurfaceBuild};
use winit::{self, MonitorId, MouseCursor};
use winit::dpi::LogicalSize;
use App;

pub use winit::WindowId as Id;

/// The default dimensions used for a window in the case that none are specified.
pub const DEFAULT_DIMENSIONS: LogicalSize = LogicalSize { width: 1024.0, height: 768.0 };

/// For building an OpenGL window.
///
/// Window parameters can be specified via the `window` method.
///
/// OpenGL context parameters can be specified via the `context` method.
pub struct Builder<'app> {
    app: &'app App,
    vulkan_physical_device: Option<PhysicalDevice<'app>>,
    window: winit::WindowBuilder,
    title_was_set: bool,
    swapchain_builder: SwapchainBuilder,
}

/// An OpenGL window.
///
/// The `Window` acts as a wrapper around the `glium::Display` type, providing a more
/// nannou-friendly API.
#[derive(Debug)]
pub struct Window {
    pub(crate) queue: Arc<device::Queue>,
    pub(crate) surface: Arc<Surface>,
    pub(crate) swapchain: Arc<WindowSwapchain>,
    pub(crate) frame_count: u64,
    // If the user specified one of the following parameters, use these when recreating the
    // swapchain rather than our heuristics.
    pub(crate) user_specified_present_mode: Option<PresentMode>,
    pub(crate) user_specified_image_count: Option<u32>,
}

/// The surface type associated with a winit window.
pub type Surface = vulkano::swapchain::Surface<winit::Window>;

/// The swapchain type associated with a winit window surface.
pub type Swapchain = vulkano::swapchain::Swapchain<winit::Window>;

/// The vulkan image type associated with a winit window surface.
pub type SwapchainImage = vulkano::image::swapchain::SwapchainImage<winit::Window>;

/// The future representing the moment that the GPU will have access to the swapchain image.
pub type SwapchainAcquireFuture = vulkano::swapchain::SwapchainAcquireFuture<winit::Window>;

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
    //
    // This is initialised to `Some(vulkano::sync::now(device))`. An `Option` is used to allow for
    // taking ownership in the application loop where we are required to join `previous_frame_end`
    // with the future associated with acquiring an image from the GPU.
    pub(crate) previous_frame_end: Mutex<Option<Box<GpuFuture>>>,
}

/// The errors that might occur while constructing a `Window`.
#[derive(Debug)]
pub enum BuildError {
    SurfaceCreation(vulkano_win::CreationError),
    DeviceCreation(vulkano::device::DeviceCreationError),
    SwapchainCreation(SwapchainCreationError),
    SwapchainCapabilities(vulkano::swapchain::CapabilitiesError),
    SurfaceDoesNotSupportCompositeAlphaOpaque,
}

/// Swapchain building parameters for which Nannou will provide a default if unspecified.
///
/// See the builder methods for more details on each parameter.
///
/// Valid parameters can be determined prior to building by checking the result of
/// [vulkano::swapchain::Surface::capabilities](https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html#method.capabilities).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwapchainBuilder {
    pub format: Option<Format>,
    pub color_space: Option<ColorSpace>,
    pub layers: Option<u32>,
    pub present_mode: Option<PresentMode>,
    pub composite_alpha: Option<CompositeAlpha>,
    pub clipped: Option<bool>,
    pub image_count: Option<u32>,
    pub surface_transform: Option<SurfaceTransform>,
}

impl SwapchainBuilder {
    pub const DEFAULT_CLIPPED: bool = true;
    pub const DEFAULT_COLOR_SPACE: ColorSpace = ColorSpace::SrgbNonLinear;
    pub const DEFAULT_COMPOSITE_ALPHA: CompositeAlpha = CompositeAlpha::Opaque;
    pub const DEFAULT_LAYERS: u32 = 1;
    pub const DEFAULT_SURFACE_TRANSFORM: SurfaceTransform = SurfaceTransform::Identity;

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
    pub fn format(mut self, format: Format) -> Self {
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
    pub fn color_space(mut self, color_space: ColorSpace) -> Self {
        self.color_space = Some(color_space);
        self
    }

    /// How the alpha values of the pixels of the window are treated.
    ///
    /// By default, nannou uses `CompositeAlpha::Opaque`.
    ///
    /// See the [vulkano docs](https://docs.rs/vulkano/latest/vulkano/swapchain/enum.CompositeAlpha.html).
    pub fn composite_alpha(mut self, composite_alpha: CompositeAlpha) -> Self {
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
    pub fn present_mode(mut self, present_mode: PresentMode) -> Self {
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
    pub fn surface_transform(mut self, surface_transform: SurfaceTransform) -> Self {
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
        device: Arc<Device>,
        surface: Arc<Surface>,
        sharing_mode: S,
        loop_mode: &LoopMode,
        fallback_dimensions: Option<[u32; 2]>,
        old_swapchain: Option<&Arc<Swapchain>>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<SwapchainImage>>), SwapchainCreationError>
    where
        S: Into<vulkano::sync::SharingMode>,
    {
        let capabilities = surface.capabilities(device.physical_device())
            .expect("failed to retrieve surface capabilities");

        let dimensions = capabilities
            .current_extent
            .or(fallback_dimensions)
            .unwrap_or([DEFAULT_DIMENSIONS.width as _, DEFAULT_DIMENSIONS.height as _]);

        // Retrieve the format.
        let format = match self.format {
            Some(fmt) => fmt,
            None => {
                let color_space = self.color_space.unwrap_or(Self::DEFAULT_COLOR_SPACE);
                capabilities
                    .supported_formats
                    .into_iter()
                    .filter(|(_, cs)| *cs == color_space)
                    .map(|(fmt, _)| fmt)
                    .next()
                    .ok_or(SwapchainCreationError::UnsupportedFormat)?
            }
        };

        // Determine the optimal present mode and image count based on the specified parameters and
        // the current loop mode.
        let (present_mode, image_count) = preferred_present_mode_and_image_count(
            &loop_mode,
            capabilities.min_image_count,
            self.present_mode,
            self.image_count,
        );

        // Attempt to retrieve the desired composite alpha.
        let composite_alpha = match self.composite_alpha {
            Some(alpha) => alpha,
            None => match capabilities.supported_composite_alpha.opaque {
                true => Self::DEFAULT_COMPOSITE_ALPHA,
                false => return Err(SwapchainCreationError::UnsupportedCompositeAlpha),
            }
        };

        let layers = self.layers.unwrap_or(Self::DEFAULT_LAYERS);
        let clipped = self.clipped.unwrap_or(Self::DEFAULT_CLIPPED);
        let surface_transform = self.surface_transform.unwrap_or(Self::DEFAULT_SURFACE_TRANSFORM);

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
    present_mode: Option<PresentMode>,
    image_count: Option<u32>,
) -> (PresentMode, u32) {
    match (present_mode, image_count) {
        (Some(pm), Some(ic)) => (pm, ic),
        (None, _) => match *loop_mode {
            LoopMode::RefreshSync { .. } => {
                let image_count = image_count.unwrap_or_else(|| {
                    cmp::max(min_image_count, 2)
                });
                (PresentMode::Fifo, image_count)
            }
            LoopMode::Wait { .. } | LoopMode::Rate { .. } => {
                let image_count = image_count.unwrap_or_else(|| {
                    cmp::max(min_image_count, 3)
                });
                (PresentMode::Mailbox, image_count)
            }
        }
        (Some(present_mode), None) => {
            let image_count = match present_mode {
                PresentMode::Immediate => min_image_count,
                PresentMode::Mailbox => cmp::max(min_image_count, 3),
                PresentMode::Fifo => cmp::max(min_image_count, 2),
                PresentMode::Relaxed => cmp::max(min_image_count, 2),
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
            vulkan_physical_device: None,
            window: winit::WindowBuilder::new(),
            title_was_set: false,
            swapchain_builder: Default::default(),
        }
    }

    /// Build the window with some custom window parameters.
    pub fn window(mut self, window: winit::WindowBuilder) -> Self {
        self.window = window;
        self
    }

    /// The physical device to associate with the window surface's swapchain.
    pub fn vulkan_physical_device(mut self, device: PhysicalDevice<'app>) -> Self {
        self.vulkan_physical_device = Some(device);
        self
    }

    /// Specify a set of parameters for building the window surface swapchain.
    pub fn swapchain_builder(mut self, swapchain_builder: SwapchainBuilder) -> Self {
        self.swapchain_builder = swapchain_builder;
        self
    }

    /// Builds the window, inserts it into the `App`'s display map and returns the unique ID.
    pub fn build(self) -> Result<Id, BuildError> {
        let Builder {
            app,
            vulkan_physical_device,
            mut window,
            title_was_set,
            swapchain_builder,
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

        // Retrieve the physical, vulkan-supported device to use.
        let physical_device = vulkan_physical_device
            .or_else(|| app.default_vulkan_physical_device())
            .unwrap_or_else(|| unimplemented!());

        // Retrieve dimensions to use as a fallback in case vulkano swapchain capabilities
        // `current_extent` is `None`. This happens when the window size is determined by the size
        // of the swapchain.
        let initial_swapchain_dimensions = window.window.dimensions
            .or_else(|| {
                window.window.fullscreen.as_ref().map(|monitor| {
                    monitor.get_dimensions().to_logical(1.0)
                })
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
        let surface = window.build_vk_surface(&app.events_loop, app.vulkan_instance.clone())?;

        // Select the queue family to use. Default to the first graphics-supporting queue.
        let queue_family = physical_device.queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .unwrap_or_else(|| unimplemented!("couldn't find a graphical queue family"));

        // We only have one queue, so give an arbitrary priority.
        let queue_priority = 0.5;

        // If a window already exists, use the same logical device queue.
        //
        // Otherwise, create the logical device describing a channel of communication with the
        // physical device.
        let (device, queue) = match app
            .windows
            .borrow()
            .values()
            .next()
            .map(|w| w.queue.clone())
        {
            Some(queue) => (queue.device().clone(), queue),
            None => {
                let device_ext = vulkano::device::DeviceExtensions {
                    khr_swapchain: true,
                    ..vulkano::device::DeviceExtensions::none()
                };
                let (device, mut queues) = Device::new(
                    physical_device,
                    physical_device.supported_features(),
                    &device_ext,
                    [(queue_family, queue_priority)].iter().cloned(),
                )?;
                // Since it is possible to request multiple queues, the queues variable returned by
                // the function is in fact an iterator. In this case this iterator contains just
                // one element, so let's extract it.
                let queue = queues.next().expect("expected a single device queue");
                (device, queue)
            }
        };

        let user_specified_present_mode = swapchain_builder.present_mode;
        let user_specified_image_count = swapchain_builder.image_count;

        // Build the swapchain used for displaying the window contents.
        let (swapchain, images) = {
            // Set the dimensions of the swapchain to that of the surface.
            let fallback_dimensions =
                [initial_swapchain_dimensions.width as _, initial_swapchain_dimensions.height as _];

            swapchain_builder.build(
                device.clone(),
                surface.clone(),
                &queue,
                &app.loop_mode(),
                Some(fallback_dimensions),
                None,
            )?
        };

        let window_id = surface.window().id();
        let needs_recreation = AtomicBool::new(false);
        let now = Box::new(vulkano::sync::now(queue.device().clone())) as Box<GpuFuture>;
        let previous_frame_end = Mutex::new(Some(now));
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
            swapchain,
            frame_count,
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
            vulkan_physical_device,
            window,
            title_was_set,
            swapchain_builder,
        } = self;
        let window = map(window);
        Builder {
            app,
            vulkan_physical_device,
            window,
            title_was_set,
            swapchain_builder,
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
    pub fn swapchain_device(&self) -> &Arc<device::Device> {
        device::DeviceOwned::device(self.swapchain())
    }

    /// The vulkan graphics queue on which the window swapchain work is run.
    pub fn swapchain_queue(&self) -> &Arc<device::Queue> {
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

    // Custom methods.

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

    /// A utility function to simplify the recreation of a swapchain.
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
            .take()
            .expect("`previous_frame_end` was `None`");
        self.swapchain = Arc::new(WindowSwapchain {
            needs_recreation: AtomicBool::new(false),
            frame_created: self.frame_count,
            swapchain: new_swapchain,
            images: new_images,
            previous_frame_end: Mutex::new(Some(previous_frame_end)),
        });
    }
}

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

impl StdError for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::SurfaceCreation(ref err) => err.description(),
            BuildError::DeviceCreation(ref err) => err.description(),
            BuildError::SwapchainCreation(ref err) => err.description(),
            BuildError::SwapchainCapabilities(ref err) => err.description(),
            BuildError::SurfaceDoesNotSupportCompositeAlphaOpaque =>
                "`CompositeAlpha::Opaque` not supported by window surface",
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<vulkano_win::CreationError> for BuildError {
    fn from(e: vulkano_win::CreationError) -> Self {
        BuildError::SurfaceCreation(e)
    }
}

impl From<vulkano::device::DeviceCreationError> for BuildError {
    fn from(e: vulkano::device::DeviceCreationError) -> Self {
        BuildError::DeviceCreation(e)
    }
}

impl From<vulkano::swapchain::SwapchainCreationError> for BuildError {
    fn from(e: vulkano::swapchain::SwapchainCreationError) -> Self {
        BuildError::SwapchainCreation(e)
    }
}

impl From<vulkano::swapchain::CapabilitiesError> for BuildError {
    fn from(e: vulkano::swapchain::CapabilitiesError) -> Self {
        BuildError::SwapchainCapabilities(e)
    }
}
