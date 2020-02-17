//! The nannou [**Window**](./struct.Window.html) API. Create a new window via `.app.new_window()`.
//! This produces a [**Builder**](./struct.Builder.html) which can be used to build a window.

use crate::app::LoopMode;
use crate::event::{
    Key, MouseButton, MouseScrollDelta, TouchEvent, TouchPhase, TouchpadPressure, WindowEvent,
};
use crate::frame::{self, Frame, RawFrame};
use crate::geom;
use crate::geom::{Point2, Vector2};
use crate::wgpu;
use crate::App;
use std::any::Any;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::{env, fmt};
use winit::dpi::LogicalSize;

pub use winit::window::WindowId as Id;

/// The default dimensions used for a window in the case that none are specified.
pub const DEFAULT_DIMENSIONS: LogicalSize<geom::scalar::Default> = LogicalSize {
    width: 1024.0,
    height: 768.0,
};

/// A context for building a window.
pub struct Builder<'app> {
    app: &'app App,
    window: winit::window::WindowBuilder,
    title_was_set: bool,
    swap_chain_builder: SwapChainBuilder,
    request_adapter_opts: Option<wgpu::RequestAdapterOptions>,
    device_desc: Option<wgpu::DeviceDescriptor>,
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
/// Unlike the `ViewFn`, the `RawViewFn` is designed for drawing directly to a window's swap chain
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
pub type RawEventFn<Model> = fn(&App, &mut Model, &winit::event::WindowEvent);

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

/// Errors that might occur while building the window.
#[derive(Debug)]
pub enum BuildError {
    NoAvailableAdapter,
    WinitOsError(winit::error::OsError),
}

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
/// The `Window` acts as a wrapper around the `winit::window::Window` and `vulkano::Surface` types and
/// manages the associated swap chain, providing a more nannou-friendly API.
#[derive(Debug)]
pub struct Window {
    pub(crate) window: winit::window::Window,
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    msaa_samples: u32,
    pub(crate) swap_chain: WindowSwapChain,
    // Data for rendering a `Frame`'s intermediary image to a swap chain image.
    pub(crate) frame_render_data: Option<frame::RenderData>,
    pub(crate) frame_count: u64,
    pub(crate) user_functions: UserFunctions,
}

/// A swap_chain and its images associated with a single window.
pub(crate) struct WindowSwapChain {
    // Tracks whether or not the swap chain needs recreation due to resizing, etc.
    pub(crate) needs_recreation: AtomicBool,
    // The descriptor used to create the original swap chain. Useful for recreation.
    pub(crate) descriptor: wgpu::SwapChainDescriptor,
    // This is an `Option` in order to allow for separating ownership of the swapchain from the
    // window during a `RedrawRequest`. Other than during `RedrawRequest`, this should always be
    // `Some`.
    pub(crate) swap_chain: Option<wgpu::SwapChain>,
}

/// SwapChain building parameters for which Nannou will provide a default if unspecified.
///
/// See the builder methods for more details on each parameter.
#[derive(Clone, Debug, Default)]
pub struct SwapChainBuilder {
    pub usage: Option<wgpu::TextureUsage>,
    pub format: Option<wgpu::TextureFormat>,
    pub present_mode: Option<wgpu::PresentMode>,
}

impl SwapChainBuilder {
    pub const DEFAULT_USAGE: wgpu::TextureUsage = wgpu::TextureUsage::OUTPUT_ATTACHMENT;
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
    pub const DEFAULT_PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::Vsync;

    /// A new empty **SwapChainBuilder** with all parameters set to `None`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a **SwapChainBuilder** from an existing descriptor.
    ///
    /// The resulting swap chain parameters will match that of the given `SwapChainDescriptor`.
    pub fn from_descriptor(desc: &wgpu::SwapChainDescriptor) -> Self {
        SwapChainBuilder::new()
            .usage(desc.usage)
            .format(desc.format)
            .present_mode(desc.present_mode)
    }

    /// Specify the texture usage for the swap chain.
    pub fn usage(mut self, usage: wgpu::TextureUsage) -> Self {
        self.usage = Some(usage);
        self
    }

    /// Specify the texture format for the swap chain.
    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// The way in which swap chain images are presented to the display.
    ///
    /// By default, nannou will attempt to select the ideal present mode depending on the current
    /// app `LoopMode`.
    pub fn present_mode(mut self, present_mode: wgpu::PresentMode) -> Self {
        self.present_mode = Some(present_mode);
        self
    }

    /// Build the swap chain.
    pub(crate) fn build(
        self,
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        [width_px, height_px]: [u32; 2],
        loop_mode: &LoopMode,
    ) -> (wgpu::SwapChain, wgpu::SwapChainDescriptor) {
        let usage = self.usage.unwrap_or(Self::DEFAULT_USAGE);
        let format = self.format.unwrap_or(Self::DEFAULT_FORMAT);
        let present_mode = self
            .present_mode
            .unwrap_or_else(|| preferred_present_mode(loop_mode));
        let desc = wgpu::SwapChainDescriptor {
            usage,
            format,
            width: width_px,
            height: height_px,
            present_mode,
        };
        let swap_chain = device.create_swap_chain(surface, &desc);
        (swap_chain, desc)
    }
}

/// Determine the optimal present mode for the given loop mode.
///
/// TODO: Currently this always assumes `Vsync`. Do we want to provide `NoVsync` option for *any*
/// loop modes?
pub fn preferred_present_mode(_loop_mode: &LoopMode) -> wgpu::PresentMode {
    wgpu::PresentMode::Vsync
}

impl<'app> Builder<'app> {
    /// Begin building a new window.
    pub fn new(app: &'app App) -> Self {
        Builder {
            app,
            window: winit::window::WindowBuilder::new(),
            title_was_set: false,
            swap_chain_builder: Default::default(),
            request_adapter_opts: None,
            device_desc: None,
            user_functions: Default::default(),
            msaa_samples: None,
        }
    }

    /// Build the window with some custom window parameters.
    pub fn window(mut self, window: winit::window::WindowBuilder) -> Self {
        self.window = window;
        self
    }

    /// Specify a set of parameters for building the window surface swap chain.
    pub fn swap_chain_builder(mut self, swap_chain_builder: SwapChainBuilder) -> Self {
        self.swap_chain_builder = swap_chain_builder;
        self
    }

    /// Specify a custom set of options to request an adapter with. This is useful for describing a
    /// set of desired properties for the requested physical device.
    pub fn request_adapter_options(mut self, opts: wgpu::RequestAdapterOptions) -> Self {
        self.request_adapter_opts = Some(opts);
        self
    }

    /// Specify a device descriptor to use when requesting the logical device from the adapter.
    /// This allows for specifying custom wgpu device extensions.
    pub fn device_descriptor(mut self, device_desc: wgpu::DeviceDescriptor) -> Self {
        self.device_desc = Some(device_desc);
        self
    }

    /// Specify the number of samples per pixel for the multisample anti-aliasing render pass.
    ///
    /// If `msaa_samples` is unspecified, the first default value that nannou will attempt to use
    /// can be found via the `Frame::DEFAULT_MSAA_SAMPLES` constant.
    ///
    /// **Note:** This parameter has no meaning if the window uses a **raw_view** function for
    /// rendering graphics to the window rather than a **view** function. This is because the
    /// **raw_view** function provides a **RawFrame** with direct access to the swap chain image
    /// itself and thus must manage their own MSAA pass.
    ///
    /// On the other hand, the `view` function provides the `Frame` type which allows the user to
    /// render to a multisampled intermediary image allowing Nannou to take care of resolving the
    /// multisampled image to the swap chain image. In order to avoid confusion, The `Window::build`
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
    /// drawing directly to a window's swap chain images, rather than to a convenient intermediary
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

    /// The same as the `event` method, but allows for processing raw `winit::event::WindowEvent`s rather
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
            mut window,
            title_was_set,
            swap_chain_builder,
            request_adapter_opts,
            device_desc,
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

        // Set default dimensions in the case that none were given.
        let initial_window_size = window
            .window
            .inner_size
            .or_else(|| {
                window
                    .window
                    .fullscreen
                    .as_ref()
                    .map(|fullscreen| match fullscreen {
                        winit::window::Fullscreen::Exclusive(video_mode) => {
                            let monitor = video_mode.monitor();
                            video_mode.size().to_logical::<f32>(monitor.scale_factor()).into()
                        }
                        winit::window::Fullscreen::Borderless(monitor) => {
                            monitor.size().to_logical::<f32>(monitor.scale_factor()).into()
                        }
                    })
            })
            .unwrap_or_else(|| {
                let mut dim = DEFAULT_DIMENSIONS;
                if let Some(min) = window.window.min_inner_size {
                    match min {
                        winit::dpi::Size::Logical(min) => {
                            dim.width = dim.width.max(min.width as _);
                            dim.height = dim.height.max(min.height as _);
                        }
                        winit::dpi::Size::Physical(min) => {
                            dim.width = dim.width.max(min.width as _);
                            dim.height = dim.height.max(min.height as _);
                            unimplemented!("consider scale factor");
                        }
                    }
                }
                if let Some(max) = window.window.max_inner_size {
                    match max {
                        winit::dpi::Size::Logical(max) => {
                            dim.width = dim.width.min(max.width as _);
                            dim.height = dim.height.min(max.height as _);
                        }
                        winit::dpi::Size::Physical(max) => {
                            dim.width = dim.width.min(max.width as _);
                            dim.height = dim.height.min(max.height as _);
                            unimplemented!("consider scale factor");
                        }
                    }
                }
                dim.into()
            });

        // Use the `initial_swapchain_dimensions` as the default dimensions for the window if none
        // were specified.
        if window.window.inner_size.is_none() && window.window.fullscreen.is_none() {
            window.window.inner_size = Some(initial_window_size);
        }

        // Build the window.
        let window = {
            let window_target = app
                .event_loop_window_target
                .as_ref()
                .expect("unexpected invalid App.event_loop_window_target state - please report")
                .as_ref();
            window.build(window_target)?
        };

        // Build the wgpu surface.
        let surface = wgpu::Surface::create(&window);

        // Request the adapter.
        let request_adapter_opts =
            request_adapter_opts.unwrap_or(wgpu::DEFAULT_ADAPTER_REQUEST_OPTIONS);
        let adapter =
            wgpu::Adapter::request(&request_adapter_opts).ok_or(BuildError::NoAvailableAdapter)?;

        // Instantiate the logical device.
        let device_desc = device_desc.unwrap_or_else(wgpu::default_device_descriptor);
        let (device, queue) = adapter.request_device(&device_desc);

        // Build the swapchain.
        let win_dims_px: [u32; 2] = window.inner_size().into();
        let (swap_chain, swap_chain_desc) =
            swap_chain_builder.build(&device, &surface, win_dims_px, &app.loop_mode());

        // If we're using an intermediary image for rendering frames to swap_chain images, create
        // the necessary render data.
        let (frame_render_data, msaa_samples) = match user_functions.view {
            Some(View::WithModel(_)) | Some(View::Sketch(_)) | None => {
                let msaa_samples = msaa_samples.unwrap_or(Frame::DEFAULT_MSAA_SAMPLES);
                // TODO: Verity that requested sample count is valid for surface?
                let swap_chain_dims = [swap_chain_desc.width, swap_chain_desc.height];
                let render_data = frame::RenderData::new(
                    &device,
                    swap_chain_dims,
                    swap_chain_desc.format,
                    msaa_samples,
                );
                (Some(render_data), msaa_samples)
            }
            Some(View::WithModelRaw(_)) => (None, 1),
        };

        let window_id = window.id();
        let needs_recreation = AtomicBool::new(false);
        let frame_count = 0;
        let swap_chain = WindowSwapChain {
            needs_recreation,
            descriptor: swap_chain_desc,
            swap_chain: Some(swap_chain),
        };

        let window = Window {
            window,
            surface,
            device,
            queue,
            msaa_samples,
            swap_chain,
            frame_render_data,
            frame_count,
            user_functions,
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
        F: FnOnce(winit::window::WindowBuilder) -> winit::window::WindowBuilder,
    {
        let Builder {
            app,
            window,
            title_was_set,
            device_desc,
            request_adapter_opts,
            swap_chain_builder,
            user_functions,
            msaa_samples,
        } = self;
        let window = map(window);
        Builder {
            app,
            window,
            title_was_set,
            device_desc,
            request_adapter_opts,
            swap_chain_builder,
            user_functions,
            msaa_samples,
        }
    }

    // Window builder methods.

    /// Requests the window to be a specific size in points.
    ///
    /// This describes to the "inner" part of the window, not including desktop decorations like the
    /// title bar.
    pub fn size(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_inner_size(winit::dpi::LogicalSize { width, height }))
    }

    /// Set the minimum size in points for the window.
    pub fn min_size(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_min_inner_size(winit::dpi::LogicalSize { width, height }))
    }

    /// Set the maximum size in points for the window.
    pub fn max_size(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_max_inner_size(winit::dpi::LogicalSize { width, height }))
    }

    /// Requests the window to be a specific size in points.
    ///
    /// This describes to the "inner" part of the window, not including desktop decorations like the
    /// title bar.
    pub fn size_pixels(self, width: u32, height: u32) -> Self {
        self.map_window(|w| w.with_inner_size(winit::dpi::PhysicalSize { width, height }))
    }

    /// Requests a specific title for the window.
    pub fn title<T>(mut self, title: T) -> Self
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
    pub fn fullscreen(self, fullscreen: Option<winit::window::Fullscreen>) -> Self {
        self.map_window(|w| w.with_fullscreen(fullscreen))
    }

    /// Requests maximized mode.
    pub fn maximized(self, maximized: bool) -> Self {
        self.map_window(|w| w.with_maximized(maximized))
    }

    /// Sets whether the window will be initially hidden or visible.
    pub fn visible(self, visible: bool) -> Self {
        self.map_window(|w| w.with_visible(visible))
    }

    /// Sets whether the background of the window should be transparent.
    pub fn transparent(self, transparent: bool) -> Self {
        self.map_window(|w| w.with_transparent(transparent))
    }

    /// Sets whether the window should have a border, a title bar, etc.
    pub fn decorations(self, decorations: bool) -> Self {
        self.map_window(|w| w.with_decorations(decorations))
    }
}

impl Window {
    // `winit::window::Window` methods.

    /// Modifies the title of the window.
    ///
    /// This is a no-op if the window has already been closed.
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Shows the window if it was hidden.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    #[deprecated(note = "please use `set_visible(true)` instead")]
    pub fn show(&self) {
        self.set_visible(true)
    }

    /// Hides the window if it was visible.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    #[deprecated(note = "please use `set_visible(false)` instead")]
    pub fn hide(&self) {
        self.set_visible(false)
    }

    /// Set the visibility of the window.
    ///
    /// ## Platform-specific
    ///
    /// - Android: Has no effect.
    /// - iOS: Can only be called on the main thread.
    /// - Web: Has no effect.
    pub fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible)
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
    pub fn outer_position_pixels(&self) -> Result<(i32, i32), winit::error::NotSupportedError> {
        self.window.outer_position().map(Into::into)
    }

    /// Modifies the position of the window.
    ///
    /// See `position` for more information about the returned coordinates.
    pub fn set_outer_position_pixels(&self, x: i32, y: i32) {
        self.window
            .set_outer_position(winit::dpi::PhysicalPosition { x, y })
    }

    /// The size in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. These
    /// are the dimensions of the frame buffer, and the dimensions that you should use when you
    /// call glViewport.
    pub fn inner_size_pixels(&self) -> (u32, u32) {
        self.window.inner_size().into()
    }

    /// The size in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders. To get
    /// the dimensions of the frame buffer when calling `glViewport`, multiply with hidpi factor.
    ///
    /// This is the same as dividing the result  of `inner_size_pixels()` by `scale_factor()`.
    pub fn inner_size_points(&self) -> (geom::scalar::Default, geom::scalar::Default) {
        self.window
            .inner_size()
            .to_logical::<f32>(self.window.scale_factor())
            .into()
    }

    /// The size of the window in pixels.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    /// `inner_size_pixels` instead.
    pub fn outer_size_pixels(&self) -> (u32, u32) {
        self.window.outer_size().into()
    }

    /// The size of the window in points.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    /// `inner_size_points` instead.
    ///
    /// This is the same as dividing the result  of `outer_size_pixels()` by `scale_factor()`.
    pub fn outer_size_points(&self) -> (f32, f32) {
        self.window
            .outer_size()
            .to_logical::<f32>(self.window.scale_factor())
            .into()
    }

    /// Modifies the inner size of the window.
    ///
    /// See the `inner_size` methods for more informations about the values.
    pub fn set_inner_size_pixels(&self, width: u32, height: u32) {
        self.window
            .set_inner_size(winit::dpi::PhysicalSize { width, height })
    }

    /// Modifies the inner size of the window using point values.
    ///
    /// Internally, the given width and height are multiplied by the `scale_factor` to get the
    /// values in pixels before calling `set_inner_size_pixels` internally.
    pub fn set_inner_size_points(&self, width: f32, height: f32) {
        self.window
            .set_inner_size(winit::dpi::LogicalSize { width, height })
    }

    /// The ratio between the backing framebuffer resolution and the window size in screen pixels.
    ///
    /// This is typically `1.0` for a normal display, `2.0` for a retina display and higher on more
    /// modern displays.
    pub fn scale_factor(&self) -> geom::scalar::Default {
        self.window.scale_factor() as _
    }

    /// Changes the position of the cursor in logical window coordinates.
    pub fn set_cursor_position_points(
        &self,
        x: f32,
        y: f32,
    ) -> Result<(), winit::error::ExternalError> {
        self.window
            .set_cursor_position(winit::dpi::LogicalPosition { x, y })
    }

    /// Modifies the mouse cursor of the window.
    ///
    /// ## Platform-specific
    ///
    /// Has no effect on Android.
    pub fn set_cursor_icon(&self, state: winit::window::CursorIcon) {
        self.window.set_cursor_icon(state);
    }

    /// Grabs the cursor, preventing it from leaving the window.
    ///
    /// ## Platform-specific
    ///
    /// On macOS, this presently merely locks the cursor in a fixed location, which looks visually
    /// awkward.
    ///
    /// This has no effect on Android or iOS.
    pub fn set_cursor_grab(&self, grab: bool) -> Result<(), winit::error::ExternalError> {
        self.window.set_cursor_grab(grab)
    }

    /// Hides the cursor with `false`, making it invisible but still usable.
    ///
    /// ## Platform-specific
    ///
    /// On Windows and X11, the cursor is only hidden within the confines of the window.
    ///
    /// On macOS, the cursor is hidden as long as the window has input focus, even if the cursor is
    /// outside of the window.
    ///
    /// This has no effect on Android or iOS.
    pub fn set_cursor_visible(&self, hide: bool) {
        self.window.set_cursor_visible(hide)
    }

    /// Sets the window to maximized or back.
    pub fn set_maximized(&self, maximized: bool) {
        self.window.set_maximized(maximized)
    }

    /// Set the window to fullscreen.
    ///
    /// Call this method again with `None` to revert back from fullscreen.
    ///
    /// ## Platform-specific
    ///
    /// - macOS: `Fullscreen::Exclusive` provides true exclusive mode with a video mode change.
    ///   Caveat! macOS doesn't provide task switching (or spaces!) while in exclusive fullscreen
    ///   mode. This mode should be used when a video mode change is desired, but for a better user
    ///   experience, borderless fullscreen might be preferred.
    ///
    ///   `Fullscreen::Borderless` provides a borderless fullscreen window on a separate space.
    ///   This is the idiomatic way for fullscreen games to work on macOS. See
    ///   WindowExtMacOs::set_simple_fullscreen if separate spaces are not preferred.
    ///
    ///   The dock and the menu bar are always disabled in fullscreen mode.
    ///
    /// - iOS: Can only be called on the main thread.
    /// - Wayland: Does not support exclusive fullscreen mode.
    /// - Windows: Screen saver is disabled in fullscreen mode.
    pub fn set_fullscreen(&self, monitor: Option<winit::window::Fullscreen>) {
        self.window.set_fullscreen(monitor)
    }

    /// The current monitor that the window is on or the primary monitor if nothing matches.
    pub fn current_monitor(&self) -> winit::monitor::MonitorHandle {
        self.window.current_monitor()
    }

    /// A unique identifier associated with this window.
    pub fn id(&self) -> Id {
        self.window.id()
    }

    // Access to vulkano API.

    /// Returns a reference to the window's Vulkan swap chain surface.
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    /// The descriptor for the swap chain associated with this window's vulkan surface.
    pub fn swap_chain_descriptor(&self) -> &wgpu::SwapChainDescriptor {
        &self.swap_chain.descriptor
    }

    /// The vulkan logical device on which the window's swap chain is running.
    ///
    /// This is shorthand for `DeviceOwned::device(window.swap_chain())`.
    pub fn swap_chain_device(&self) -> &wgpu::Device {
        &self.device
    }

    /// The vulkan graphics queue on which the window swap chain work is run.
    pub fn swap_chain_queue(&self) -> &wgpu::Queue {
        &self.queue
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

    // A utility function to simplify the recreation of a swap_chain.
    pub(crate) fn replace_swap_chain(
        &mut self,
        new_descriptor: wgpu::SwapChainDescriptor,
        new_swap_chain: wgpu::SwapChain,
    ) {
        self.swap_chain = WindowSwapChain {
            needs_recreation: AtomicBool::new(false),
            descriptor: new_descriptor,
            swap_chain: Some(new_swap_chain),
        };
        // TODO: Update frame_render_data? Should recreate `msaa_texture`s with new sc descriptor.
        unimplemented!();

        // let swap_chain_dims = [new_descriptor.width, new_descriptor.height];
        // self.frame_render_data = frame::RenderData::new(
        //     &self.device,
        //     swap_chain_dims,
        //     self.msaa_samples
        // );
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
        let (mw, mh): (u32, u32) = self.current_monitor().size().into();
        w == mw && h == mh
    }

    /// The number of times `view` has been called with a `Frame` for this window.
    pub fn elapsed_frames(&self) -> u64 {
        self.frame_count
    }

    /// The rectangle representing the position and dimensions of the window.
    ///
    /// The window's position will always be `[0.0, 0.0]`, as positions are generally described
    /// relative to the centre of the window itself.
    ///
    /// The dimensions will be equal to the result of `inner_size_points`. This represents the area
    /// of the that we can draw to in a DPI-agnostic manner, typically useful for drawing and UI
    /// positioning.
    pub fn rect(&self) -> geom::Rect {
        let (w, h) = self.inner_size_points();
        geom::Rect::from_w_h(w, h)
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

impl fmt::Debug for WindowSwapChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WindowSwapChain ( descriptor: {:?}, swap_chain: {:?} )",
            self.descriptor, self.swap_chain,
        )
    }
}

// Error implementations.

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BuildError::NoAvailableAdapter => write!(f, "no available wgpu adapter detected"),
            BuildError::WinitOsError(ref e) => e.fmt(f),
        }
    }
}

impl From<winit::error::OsError> for BuildError {
    fn from(e: winit::error::OsError) -> Self {
        BuildError::WinitOsError(e)
    }
}
