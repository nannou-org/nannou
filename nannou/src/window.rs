//! The nannou Window API.
//!
//! Create a new window via `app.new_window()`. This produces a [**Builder**](./struct.Builder.html)
//! which can be used to build a [**Window**](./struct.Window.html).

use crate::color::IntoLinSrgba;
use crate::event::{
    Key, MouseButton, MouseScrollDelta, TouchEvent, TouchPhase, TouchpadPressure, WindowEvent,
};
use crate::frame::{self, Frame, RawFrame};
use crate::geom;
use crate::geom::Point2;
use crate::glam::Vec2;
use crate::wgpu;
use crate::App;
use std::any::Any;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::{env, fmt};
use winit::dpi::{LogicalSize, PhysicalSize};

pub use winit::window::Fullscreen;
pub use winit::window::WindowId as Id;

/// The default dimensions used for a window in the case that none are specified.
pub const DEFAULT_DIMENSIONS: LogicalSize<geom::scalar::Default> = LogicalSize {
    width: 1024.0,
    height: 768.0,
};

/// The default minimum dimensions used for the surface
pub const MIN_SC_PIXELS: PhysicalSize<u32> = PhysicalSize {
    width: 2,
    height: 2,
};

/// A context for building a window.
pub struct Builder<'app> {
    app: &'app App,
    window: winit::window::WindowBuilder,
    title_was_set: bool,
    surface_conf_builder: SurfaceConfigurationBuilder,
    power_preference: wgpu::PowerPreference,
    force_fallback_adapter: bool,
    device_desc: Option<wgpu::DeviceDescriptor<'static>>,
    user_functions: UserFunctions,
    msaa_samples: Option<u32>,
    max_capture_frame_jobs: u32,
    capture_frame_timeout: Option<Duration>,
    clear_color: Option<wgpu::Color>,
}

/// For storing all user functions within the window.
#[derive(Debug, Default)]
pub(crate) struct UserFunctions {
    pub(crate) view: Option<View>,
    pub(crate) event: Option<EventFnAny>,
    pub(crate) raw_event: Option<RawEventFnAny>,
    pub(crate) key_pressed: Option<KeyPressedFnAny>,
    pub(crate) key_released: Option<KeyReleasedFnAny>,
    pub(crate) received_character: Option<ReceivedCharacterFnAny>,
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
pub type ViewFn<Model> = fn(&App, &Model, Frame);

/// The user function type for drawing their model to the surface of a single window.
///
/// Unlike the `ViewFn`, the `RawViewFn` is designed for drawing directly to a window's surface
/// texture rather than to a convenient intermediary image.
pub type RawViewFn<Model> = fn(&App, &Model, RawFrame);

/// The same as `ViewFn`, but provides no user model to draw from.
///
/// Useful for simple, stateless sketching.
pub type SketchFn = fn(&App, Frame);

/// The user's view function, whether with a model or without one.
#[derive(Clone)]
pub(crate) enum View {
    WithModel(ViewFnAny),
    WithModelRaw(RawViewFnAny),
    Sketch(SketchFn),
}

/// A function for processing raw winit window events.
pub type RawEventFn<Model> = fn(&App, &mut Model, &winit::event::WindowEvent, Id);

/// A function for processing window events.
pub type EventFn<Model> = fn(&App, &mut Model, WindowEvent, Id);

/// A function for processing key press events.
pub type KeyPressedFn<Model> = fn(&App, &mut Model, Key);

/// A function for processing key release events.
pub type KeyReleasedFn<Model> = fn(&App, &mut Model, Key);

/// A function for processing received characters.
pub type ReceivedCharacterFn<Model> = fn(&App, &mut Model, char);

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
pub type MovedFn<Model> = fn(&App, &mut Model, Vec2);

/// A function for processing window resized events.
pub type ResizedFn<Model> = fn(&App, &mut Model, Vec2);

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
fn_any!(ReceivedCharacterFn<M>, ReceivedCharacterFnAny);
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
/// The **Window** acts as a wrapper around the `winit::window::Window` and the `wgpu::Surface`
/// types.
#[derive(Debug)]
pub struct Window {
    pub(crate) window: winit::window::Window,
    pub(crate) surface: wgpu::Surface,
    pub(crate) surface_conf: wgpu::SurfaceConfiguration,
    pub(crate) device_queue_pair: Arc<wgpu::DeviceQueuePair>,
    msaa_samples: u32,
    pub(crate) frame_data: Option<FrameData>,
    pub(crate) frame_count: u64,
    pub(crate) user_functions: UserFunctions,
    pub(crate) tracked_state: TrackedState,
    pub(crate) is_invalidated: bool, // Whether framebuffer must be cleared
    pub(crate) clear_color: wgpu::Color,
}

// Data related to `Frame`s produced for this window's surface textures.
#[derive(Debug)]
pub(crate) struct FrameData {
    // Data for rendering a `Frame`'s intermediary image to a surface texture.
    pub(crate) render: frame::RenderData,
    // Data for capturing a `Frame`'s intermediary image before submission.
    pub(crate) capture: frame::CaptureData,
}

// Track and store some information about the window in order to avoid making repeated internal
// queries to the platform-specific API. This is beneficial in some cases where queries to the
// platform-specific API can be very slow (e.g. macOS cocoa).
#[derive(Debug)]
pub(crate) struct TrackedState {
    // Updated on `ScaleFactorChanged`.
    pub(crate) scale_factor: f64,
    // Updated on `Resized`.
    pub(crate) physical_size: winit::dpi::PhysicalSize<u32>,
}

/// Surface configuration for which nannou will provide a default if unspecified.
///
/// See the builder methods for more details on each parameter.
#[derive(Clone, Debug, Default)]
pub struct SurfaceConfigurationBuilder {
    pub usage: Option<wgpu::TextureUsages>,
    pub format: Option<wgpu::TextureFormat>,
    pub present_mode: Option<wgpu::PresentMode>,
}

impl SurfaceConfigurationBuilder {
    /// Only used in the case that the `wgpu::Surface::get_preferred_format` method returns `None`.
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
    pub const DEFAULT_PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::Fifo;
    pub const DEFAULT_USAGE: wgpu::TextureUsages = wgpu::TextureUsages::RENDER_ATTACHMENT;

    /// A new empty **SurfaceConfigurationBuilder** with all parameters set to `None`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a **SurfaceConfigurationBuilder** from an existing descriptor.
    pub fn from_configuration(conf: &wgpu::SurfaceConfiguration) -> Self {
        SurfaceConfigurationBuilder::new()
            .usage(conf.usage)
            .format(conf.format)
            .present_mode(conf.present_mode)
    }

    /// Specify the texture usages for the surface.
    pub fn usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = Some(usage);
        self
    }

    /// Specify the texture format for the surface.
    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// The way in which a surface's frames are presented to the display.
    ///
    /// By default, nannou will attempt to select the ideal present mode depending on the current
    /// app `LoopMode`.
    pub fn present_mode(mut self, present_mode: wgpu::PresentMode) -> Self {
        self.present_mode = Some(present_mode);
        self
    }

    /// Build the surface configuration.
    pub(crate) fn build(
        self,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        [width_px, height_px]: [u32; 2],
    ) -> wgpu::SurfaceConfiguration {
        let usage = self.usage.unwrap_or(Self::DEFAULT_USAGE);
        let format = self
            .format
            .or_else(|| surface.get_preferred_format(&adapter))
            .unwrap_or(Self::DEFAULT_FORMAT);
        let present_mode = self.present_mode.unwrap_or(Self::DEFAULT_PRESENT_MODE);
        wgpu::SurfaceConfiguration {
            usage,
            format,
            width: width_px,
            height: height_px,
            present_mode,
        }
    }
}

impl<'app> Builder<'app> {
    /// The default power preference used to request the WGPU adapter.
    pub const DEFAULT_POWER_PREFERENCE: wgpu::PowerPreference = wgpu::DEFAULT_POWER_PREFERENCE;
    /// The default `force_fallback_adapter` field used to request the WGPU adapter.
    pub const DEFAULT_FORCE_FALLBACK_ADAPTER: bool = false;

    /// Begin building a new window.
    pub fn new(app: &'app App) -> Self {
        Builder {
            app,
            window: winit::window::WindowBuilder::new(),
            title_was_set: false,
            surface_conf_builder: Default::default(),
            power_preference: Self::DEFAULT_POWER_PREFERENCE,
            force_fallback_adapter: Self::DEFAULT_FORCE_FALLBACK_ADAPTER,
            device_desc: None,
            user_functions: Default::default(),
            msaa_samples: None,
            max_capture_frame_jobs: Default::default(),
            capture_frame_timeout: Default::default(),
            clear_color: None,
        }
    }

    /// Build the window with some custom window parameters.
    pub fn window(mut self, window: winit::window::WindowBuilder) -> Self {
        self.window = window;
        self
    }

    /// Specify a set of parameters for building the window surface.
    pub fn surface_conf_builder(
        mut self,
        surface_conf_builder: SurfaceConfigurationBuilder,
    ) -> Self {
        self.surface_conf_builder = surface_conf_builder;
        self
    }

    /// Specify the power preference desired for the WGPU adapter.
    ///
    /// By default, this is `wgpu::PowerPreference::HighPerformance`.
    pub fn power_preference(mut self, pref: wgpu::PowerPreference) -> Self {
        self.power_preference = pref;
        self
    }

    /// Indicates that only a fallback adapter can be returned. This is generally a "software"
    /// implementation on the system..
    ///
    /// By default, this is `false`.
    pub fn force_fallback_adapter(mut self, force: bool) -> Self {
        self.force_fallback_adapter = force;
        self
    }

    /// Specify a device descriptor to use when requesting the logical device from the adapter.
    /// This allows for specifying custom wgpu device extensions.
    pub fn device_descriptor(mut self, device_desc: wgpu::DeviceDescriptor<'static>) -> Self {
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
    /// **raw_view** function provides a **RawFrame** with direct access to the surface texture
    /// itself and thus must manage their own MSAA pass.
    ///
    /// On the other hand, the `view` function provides the `Frame` type which allows the user to
    /// render to a multisampled intermediary image allowing Nannou to take care of resolving the
    /// multisampled texture to the surface texture. In order to avoid confusion, The
    /// `Window::build` method will `panic!` if the user tries to specify `msaa_samples` as well as
    /// a `raw_view` method.
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
    /// drawing directly to a window's surface texture, rather than to a convenient intermediary
    /// image.
    pub fn raw_view<M>(mut self, raw_view_fn: RawViewFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.view = Some(View::WithModelRaw(RawViewFnAny::from_fn_ptr(raw_view_fn)));
        self
    }

    /// Set the initial color of the window background
    /// when its contents are invalidated, e.g. upon window resize.
    pub fn clear_color<C>(mut self, color: C) -> Self
    where
        C: IntoLinSrgba<f32>,
    {
        let lin_srgba = color.into_lin_srgba();
        let (r, g, b, a) = lin_srgba.into_components();
        let (r, g, b, a) = (r as f64, g as f64, b as f64, a as f64);

        self.clear_color = Some(wgpu::Color { r, g, b, a });
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

    pub fn received_character<M>(mut self, f: ReceivedCharacterFn<M>) -> Self
    where
        M: 'static,
    {
        self.user_functions.received_character = Some(ReceivedCharacterFnAny::from_fn_ptr(f));
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

    /// The maximum number of simultaneous capture frame jobs that can be run for this window
    /// before we block and wait for the existing jobs to complete.
    ///
    /// A "capture frame job" refers to the combind process of waiting to read a frame from the GPU
    /// and then writing that frame to an image file on the disk. Each call to
    /// `window.capture_frame(path)` spawns a new "capture frame job" on an internal thread pool.
    ///
    /// By default, this value is equal to the number of physical cpu threads available on the
    /// system. However, keep in mind that this means there must be room in both RAM and VRAM for
    /// this number of textures to exist at any moment in time. If you run into an "out of memory"
    /// error, try reducing the number of max jobs to a lower value, though never lower than `1`.
    ///
    /// **Panics** if the specified value is less than `1`.
    pub fn max_capture_frame_jobs(mut self, max_jobs: u32) -> Self {
        assert!(
            max_jobs >= 1,
            "must allow for at least one capture frame job at a time"
        );
        self.max_capture_frame_jobs = max_jobs;
        self
    }

    /// In the case that `max_capture_frame_jobs` is reached and the main thread must block, this
    /// specifies how long to wait for a running capture job to complete. See the
    /// `max_capture_frame_jobs` docs for more details.
    ///
    /// By default, the timeout used is equal to `app::Builder::DEFAULT_CAPTURE_FRAME_TIMEOUT`.
    ///
    /// If `None` is specified, the capture process will never time out. This may be necessary on
    /// extremely low-powered machines that take a long time to write each frame to disk.
    pub fn capture_frame_timeout(mut self, timeout: Option<std::time::Duration>) -> Self {
        self.capture_frame_timeout = timeout;
        self
    }

    #[cfg(not(target_os = "unknown"))]
    /// Builds the window, inserts it into the `App`'s display map and returns the unique ID.
    pub fn build(self) -> Result<Id, BuildError> {
        async_std::task::block_on(self.build_async())
    }

    pub async fn build_async(self) -> Result<Id, BuildError> {
        let Builder {
            app,
            mut window,
            title_was_set,
            surface_conf_builder,
            power_preference,
            force_fallback_adapter,
            device_desc,
            user_functions,
            msaa_samples,
            max_capture_frame_jobs,
            capture_frame_timeout,
            clear_color,
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

        // Set the class type for X11 if WindowExtUnix trait is compiled in winit
        // (see lines https://docs.rs/winit/0.26.0/src/winit/platform/unix.rs.html#1-7)
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use winit::platform::unix::WindowBuilderExtUnix;
            window = window.with_class("nannou".to_string(), "nannou".to_string());
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
                    .and_then(|fullscreen| match fullscreen {
                        Fullscreen::Exclusive(video_mode) => {
                            let monitor = video_mode.monitor();
                            Some(
                                video_mode
                                    .size()
                                    .to_logical::<f32>(monitor.scale_factor())
                                    .into(),
                            )
                        }
                        Fullscreen::Borderless(monitor) => monitor.as_ref().map(|monitor| {
                            monitor
                                .size()
                                .to_logical::<f32>(monitor.scale_factor())
                                .into()
                        }),
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

        // Use the `initial_window_size` as the default dimensions for the window if none
        // were specified.
        if window.window.inner_size.is_none() && window.window.fullscreen.is_none() {
            window.window.inner_size = Some(initial_window_size);
        }

        // Set a default minimum window size for configuring the surface.
        if window.window.min_inner_size.is_none() && window.window.fullscreen.is_none() {
            window.window.min_inner_size = Some(winit::dpi::Size::Physical(MIN_SC_PIXELS));
        }

        // Background must be initially cleared
        let is_invalidated = true;

        let clear_color = clear_color.unwrap_or_else(|| {
            let mut color: wgpu::Color = Default::default();
            color.a = if window.window.transparent { 0.0 } else { 1.0 };
            color
        });

        // Build the window.
        let window = {
            let window_target = app
                .event_loop_window_target
                .as_ref()
                .expect("unexpected invalid App.event_loop_window_target state - please report")
                .as_ref();
            window.build(window_target)?
        };

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas();

            web_sys::window()
                .expect("window")
                .document()
                .expect("document")
                .body()
                .expect("body")
                .append_child(&canvas)
                .expect("append_child");
        }

        // Build the wgpu surface.
        let surface = unsafe { app.instance().create_surface(&window) };

        // Request the adapter.
        let request_adapter_opts = wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: Some(&surface),
            force_fallback_adapter,
        };
        let adapter = app
            .wgpu_adapters()
            .get_or_request_async(request_adapter_opts, app.instance())
            .await
            .ok_or(BuildError::NoAvailableAdapter)?;

        // Instantiate the logical device.
        let device_desc = device_desc.unwrap_or_else(wgpu::default_device_descriptor);
        let device_queue_pair = adapter.get_or_request_device_async(device_desc).await;

        // Configure the surface.
        let win_physical_size = window.inner_size();
        let win_dims_px: [u32; 2] = win_physical_size.into();
        let device = device_queue_pair.device();
        let surface_conf = surface_conf_builder.build(&surface, &*adapter, win_dims_px);
        surface.configure(&device, &surface_conf);

        // If we're using an intermediary image for rendering frames to surface textures, create
        // the necessary render data.
        let (frame_data, msaa_samples) = match user_functions.view {
            Some(View::WithModel(_)) | Some(View::Sketch(_)) | None => {
                let msaa_samples = msaa_samples.unwrap_or(Frame::DEFAULT_MSAA_SAMPLES);
                // TODO: Verity that requested sample count is valid for surface?
                let surface_dims = [surface_conf.width, surface_conf.height];
                let render = frame::RenderData::new(
                    &device,
                    surface_dims,
                    surface_conf.format,
                    msaa_samples,
                );
                let capture =
                    frame::CaptureData::new(max_capture_frame_jobs, capture_frame_timeout);
                let frame_data = FrameData { render, capture };
                (Some(frame_data), msaa_samples)
            }
            Some(View::WithModelRaw(_)) => (None, 1),
        };

        let window_id = window.id();
        let frame_count = 0;

        let tracked_state = TrackedState {
            scale_factor: window.scale_factor(),
            physical_size: win_physical_size,
        };

        let window = Window {
            window,
            surface,
            surface_conf,
            device_queue_pair,
            msaa_samples,
            frame_data,
            frame_count,
            user_functions,
            tracked_state,
            is_invalidated,
            clear_color,
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
            power_preference,
            force_fallback_adapter,
            surface_conf_builder,
            user_functions,
            msaa_samples,
            max_capture_frame_jobs,
            capture_frame_timeout,
            clear_color,
        } = self;
        let window = map(window);
        Builder {
            app,
            window,
            title_was_set,
            device_desc,
            power_preference,
            force_fallback_adapter,
            surface_conf_builder,
            user_functions,
            msaa_samples,
            max_capture_frame_jobs,
            capture_frame_timeout,
            clear_color,
        }
    }

    // Window builder methods.
    //
    // NOTE: On new versions of winit, we should check whether or not new `WindowBuilder` methods
    // have been added that we should expose.

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

    /// Whether or not the window should be resizable after creation.
    pub fn resizable(self, resizable: bool) -> Self {
        self.map_window(|w| w.with_resizable(resizable))
    }

    /// Requests a specific title for the window.
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title_was_set = true;
        self.map_window(|w| w.with_title(title))
    }

    /// Create the window fullscreened on the current monitor.
    pub fn fullscreen(self) -> Self {
        let fullscreen = Fullscreen::Borderless(self.app.primary_monitor());
        self.fullscreen_with(Some(fullscreen))
    }

    /// Set the window fullscreen state with the given settings.
    ///
    /// - `None` indicates a normal window. This is the default case.
    /// - `Some(Fullscreen)` means fullscreen with the desired settings.
    pub fn fullscreen_with(self, fullscreen: Option<Fullscreen>) -> Self {
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

    /// Sets whether or not the window will always be on top of other windows.
    pub fn always_on_top(self, always_on_top: bool) -> Self {
        self.map_window(|w| w.with_always_on_top(always_on_top))
    }

    /// Sets the window icon.
    pub fn window_icon(self, window_icon: Option<winit::window::Icon>) -> Self {
        self.map_window(|w| w.with_window_icon(window_icon))
    }
}

impl Window {
    // `winit::window::Window` methods.
    //
    // NOTE: On new versions of winit, we should check whether or not new `Window` methods have
    // been added that we should expose. Most of the following method docs are copied from the
    // winit documentation. It would be nice if we could automate this inlining somehow.

    /// A unique identifier associated with this window.
    pub fn id(&self) -> Id {
        self.window.id()
    }

    /// Returns the scale factor that can be used to map logical pixels to physical pixels and vice
    /// versa.
    ///
    /// Throughout nannou, you will see "logical pixels" referred to as "points", and "physical
    /// pixels" referred to as "pixels".
    ///
    /// This is typically `1.0` for a normal display, `2.0` for a retina display and higher on more
    /// modern displays.
    ///
    /// You can read more about what this scale factor means within winit's [dpi module
    /// documentation](https://docs.rs/winit/latest/winit/dpi/index.html).
    ///
    /// ## Platform-specific
    ///
    /// - **X11:** This respects Xft.dpi, and can be overridden using the `WINIT_X11_SCALE_FACTOR`
    ///   environment variable.
    /// - **Android:** Always returns 1.0.
    /// - **iOS:** Can only be called on the main thread. Returns the underlying `UiView`'s
    ///   `contentScaleFactor`.
    pub fn scale_factor(&self) -> geom::scalar::Default {
        self.window.scale_factor() as _
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
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Returns the top left coordinates of the
    /// window in the screen space coordinate system.
    /// - **Web:** Returns the top-left coordinates relative to the viewport.
    pub fn outer_position_pixels(&self) -> Result<(i32, i32), winit::error::NotSupportedError> {
        self.window.outer_position().map(Into::into)
    }

    /// Modifies the position of the window.
    ///
    /// See `outer_position_pixels` for more information about the returned coordinates. This
    /// automatically un-maximizes the window if it is maximized.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Sets the top left coordinates of the
    ///   window in the screen space coordinate system.
    /// - **Web:** Sets the top-left coordinates relative to the viewport.
    pub fn set_outer_position_pixels(&self, x: i32, y: i32) {
        self.window
            .set_outer_position(winit::dpi::PhysicalPosition { x, y })
    }

    /// The width and height in pixels of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    pub fn inner_size_pixels(&self) -> (u32, u32) {
        self.window.inner_size().into()
    }

    /// The width and height in points of the client area of the window.
    ///
    /// The client area is the content of the window, excluding the title bar and borders.
    ///
    /// This is the same as dividing the result  of `inner_size_pixels()` by `scale_factor()`.
    pub fn inner_size_points(&self) -> (geom::scalar::Default, geom::scalar::Default) {
        self.window
            .inner_size()
            .to_logical::<f32>(self.tracked_state.scale_factor)
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
    /// See the `inner_size` methods for more informations about the values.
    pub fn set_inner_size_points(&self, width: f32, height: f32) {
        self.window
            .set_inner_size(winit::dpi::LogicalSize { width, height })
    }

    /// The width and height of the window in pixels.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    /// `inner_size_pixels` instead.
    pub fn outer_size_pixels(&self) -> (u32, u32) {
        self.window.outer_size().into()
    }

    /// The width and height of the window in points.
    ///
    /// These dimensions include title bar and borders. If you don't want these, you should use
    /// `inner_size_points` instead.
    ///
    /// This is the same as dividing the result  of `outer_size_pixels()` by `scale_factor()`.
    pub fn outer_size_points(&self) -> (f32, f32) {
        self.window
            .outer_size()
            .to_logical::<f32>(self.tracked_state.scale_factor)
            .into()
    }

    /// Sets a minimum size for the window.
    pub fn set_min_inner_size_points(&self, size: Option<(f32, f32)>) {
        let size = size.map(|(width, height)| winit::dpi::LogicalSize { width, height });
        self.window.set_min_inner_size(size)
    }

    /// Sets a maximum size for the window.
    pub fn set_max_inner_size_points(&self, size: Option<(f32, f32)>) {
        let size = size.map(|(width, height)| winit::dpi::LogicalSize { width, height });
        self.window.set_max_inner_size(size)
    }

    /// Modifies the title of the window.
    ///
    /// This is a no-op if the window has already been closed.
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
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

    /// Sets whether the window is resizable or not.
    ///
    /// Note that making the window unresizable doesn't exempt you from handling **Resized**, as
    /// that event can still be triggered by DPI scaling, entering fullscreen mode, etc.
    pub fn set_resizable(&self, resizable: bool) {
        self.window.set_resizable(resizable)
    }

    /// Sets the window to minimized or back.
    pub fn set_minimized(&self, minimized: bool) {
        self.window.set_minimized(minimized)
    }

    /// Sets the window to maximized or back.
    pub fn set_maximized(&self, maximized: bool) {
        self.window.set_maximized(maximized)
    }

    /// Set the window to fullscreen on the primary monitor.
    ///
    /// `true` enables fullscreen, `false` disables fullscreen.
    ///
    /// See the `set_fullscreen_with` method for more options and details about behaviour related
    /// to fullscreen.
    pub fn set_fullscreen(&self, fullscreen: bool) {
        if fullscreen {
            let monitor = self.current_monitor();
            let fullscreen = Fullscreen::Borderless(monitor);
            self.set_fullscreen_with(Some(fullscreen));
        } else {
            self.set_fullscreen_with(None);
        }
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
    pub fn set_fullscreen_with(&self, fullscreen: Option<Fullscreen>) {
        self.window.set_fullscreen(fullscreen)
    }

    /// Gets the window's current fullscreen state.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread.
    pub fn fullscreen(&self) -> Option<Fullscreen> {
        self.window.fullscreen()
    }

    /// Turn window decorations on or off.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Can only be called on the main thread. Controls whether the status bar is hidden
    ///   via `setPrefersStatusBarHidden`.
    /// - **Web:** Has no effect.
    pub fn set_decorations(&self, decorations: bool) {
        self.window.set_decorations(decorations)
    }

    /// Change whether or not the window will always be on top of other windows.
    pub fn set_always_on_top(&self, always_on_top: bool) {
        self.window.set_always_on_top(always_on_top)
    }

    /// Sets the window icon. On Windows and X11, this is typically the small icon in the top-left
    /// corner of the titlebar.
    ///
    /// ## Platform-specific
    ///
    /// This only has effect on Windows and X11.
    ///
    /// On Windows, this sets ICON_SMALL. The base size for a window icon is 16x16, but it's
    /// recommended to account for screen scaling and pick a multiple of that, i.e. 32x32.
    ///
    /// X11 has no universal guidelines for icon sizes, so you're at the whims of the WM. That
    /// said, it's usually in the same ballpark as on Windows.
    pub fn set_window_icon(&self, window_icon: Option<winit::window::Icon>) {
        self.window.set_window_icon(window_icon)
    }

    /// Sets the location of IME candidate box in client area coordinates relative to the top left.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Has no effect.
    /// - **Web:** Has no effect.
    pub fn set_ime_position_points(&self, x: f32, y: f32) {
        self.window
            .set_ime_position(winit::dpi::LogicalPosition { x, y })
    }

    /// Modifies the mouse cursor of the window.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Has no effect.
    /// - **Android:** Has no effect.
    pub fn set_cursor_icon(&self, state: winit::window::CursorIcon) {
        self.window.set_cursor_icon(state);
    }

    /// Changes the position of the cursor in logical window coordinates.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS:** Always returns an `Err`.
    /// - **Web:** Has no effect.
    pub fn set_cursor_position_points(
        &self,
        x: f32,
        y: f32,
    ) -> Result<(), winit::error::ExternalError> {
        self.window
            .set_cursor_position(winit::dpi::LogicalPosition { x, y })
    }

    /// Grabs the cursor, preventing it from leaving the window.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Locks the cursor in a fixed location.
    /// - **Wayland:** Locks the cursor in a fixed location.
    /// - **Android:** Has no effect.
    /// - **iOS:** Always returns an Err.
    /// - **Web:** Has no effect.
    pub fn set_cursor_grab(&self, grab: bool) -> Result<(), winit::error::ExternalError> {
        self.window.set_cursor_grab(grab)
    }

    /// Set the cursor's visibility.
    ///
    /// If `false`, hides the cursor. If `true`, shows the cursor.
    ///
    /// ## Platform-specific
    ///
    /// On **Windows**, **X11** and **Wayland**, the cursor is only hidden within the confines of
    /// the window.
    ///
    /// On **macOS**, the cursor is hidden as long as the window has input focus, even if the
    /// cursor is outside of the window.
    ///
    /// This has no effect on **Android** or **iOS**.
    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }

    /// The current monitor that the window is, on or the primary monitor if nothing matches.
    /// If there's neither a current nor a primary monitor, returns none.
    pub fn current_monitor(&self) -> Option<winit::monitor::MonitorHandle> {
        self.window.current_monitor()
    }

    // Access to wgpu API.

    /// Returns a reference to the window's wgpu surface.
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    /// The current configuration of the window's wgpu surface.
    pub fn surface_configuration(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_conf
    }

    /// The wgpu logical device on which the window's wgpu surface is running.
    pub fn device(&self) -> &wgpu::Device {
        self.device_queue_pair.device()
    }

    /// The wgpu graphics queue to which the window's wgpu surface frames are submitted.
    pub fn queue(&self) -> &wgpu::Queue {
        self.device_queue_pair.queue()
    }

    /// Provides access to the device queue pair and the `Arc` behind which it is stored. This can
    /// be useful in cases where using references provided by the `device` or `queue` methods cause
    /// awkward ownership problems.
    pub fn device_queue_pair(&self) -> &Arc<wgpu::DeviceQueuePair> {
        &self.device_queue_pair
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

    // A utility function to simplify the reconfiguration of the window's wgpu surface.
    //
    // Upon resizing of the window, the window's surface needs to be reconfigured to match.
    //
    // Also syncs the window tracked size with the surface size.
    pub(crate) fn reconfigure_surface(&mut self, [w_px, h_px]: [u32; 2]) {
        self.tracked_state.physical_size.width = w_px.max(MIN_SC_PIXELS.width);
        self.tracked_state.physical_size.height = h_px.max(MIN_SC_PIXELS.height);
        self.surface_conf.width = self.tracked_state.physical_size.width;
        self.surface_conf.height = self.tracked_state.physical_size.height;
        self.surface.configure(self.device(), &self.surface_conf);
        if self.frame_data.is_some() {
            let render_data = frame::RenderData::new(
                self.device(),
                self.tracked_state.physical_size.into(),
                self.surface_conf.format,
                self.msaa_samples,
            );
            self.frame_data.as_mut().unwrap().render = render_data;
        }

        // May contain uninitialized or previous contents, so must be cleared
        self.is_invalidated = true;
    }

    /// Attempts to determine whether or not the window is currently fullscreen.
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen().is_some()
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

    /// Capture the next frame right before it is drawn to this window and write it to an image
    /// file at the given path. If a frame already exists, it will be captured before its `submit`
    /// method is called or before it is `drop`ped.
    ///
    /// The destination image file type will be inferred from the extension given in the path.
    pub fn capture_frame<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        self.capture_frame_inner(path.as_ref());
    }

    /// Produces a reference to the inner winit window.
    ///
    /// This is sometimes useful for integration with other winit-aware libraries (e.g. UI).
    pub fn winit_window(&self) -> &winit::window::Window {
        &self.window
    }

    fn capture_frame_inner(&self, path: &Path) {
        // If the parent directory does not exist, create it.
        let dir = path.parent().expect("capture_frame path has no directory");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).expect("failed to create `capture_frame` directory");
        }

        let mut capture_next_frame_path = self
            .frame_data
            .as_ref()
            .expect("window capture requires that `view` draws to a `Frame` (not a `RawFrame`)")
            .capture
            .next_frame_path
            .lock()
            .expect("failed to lock `capture_next_frame_path`");
        *capture_next_frame_path = Some(path.to_path_buf());
    }

    /// Block and wait for all active capture frame jobs to complete.
    ///
    /// This is called implicitly when the window is dropped to ensure any pending captures
    /// complete.
    pub fn await_capture_frame_jobs(
        &self,
    ) -> Result<(), wgpu::TextureCapturerAwaitWorkerTimeout<()>> {
        if let Some(frame_data) = self.frame_data.as_ref() {
            let capture_data = &frame_data.capture;
            let device = self.device();
            return capture_data.texture_capturer.await_active_snapshots(device);
        }
        Ok(())
    }
}

// Drop implementations.

impl Drop for Window {
    fn drop(&mut self) {
        if self.await_capture_frame_jobs().is_err() {
            // TODO: Replace eprintlns with proper logging.
            eprintln!("timed out while waiting for capture jobs to complete");
        }
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

// Some WGPU helper implementations.

impl<'a> wgpu::WithDeviceQueuePair for &'a crate::window::Window {
    fn with_device_queue_pair<F, O>(self, f: F) -> O
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue) -> O,
    {
        self.device_queue_pair().with_device_queue_pair(f)
    }
}
