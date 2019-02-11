//! Items related to the `App` type and the application context in general.
//!
//! See here for items relating to the event loop, device access, creating and managing windows,
//! streams and more.
//!
//! - [**App**](./struct.App.html) - provides a context and API for windowing, audio, devices, etc.
//! - [**Proxy**](./struct.Proxy.html) - a handle to an **App** that may be used from a non-main
//!   thread.
//! - [**Audio**](./struct.Audio.html) - an API accessed via `app.audio` for enumerating audio
//!   devices, spawning audio input/output streams, etc.
//! - [**Draw**](./struct.Draw.html) - a simple API for drawing graphics, accessible via
//!   `app.draw()`.
//! - [**LoopMode**](./enum.LoopMode.html) - describes the behaviour of the application event loop.

use audio;
use audio::cpal;
use draw;
use event::{self, Event, LoopEvent, Key, Update};
use find_folder;
use frame::Frame;
use geom;
use gpu;
use state;
use std;
use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::atomic::{self, AtomicBool};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};
use time::DurationF64;
use ui;
use vulkano;
use vulkano::device::DeviceOwned;
use vulkano::format::Format;
use vulkano::instance::InstanceExtensions;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::GpuFuture;
use window::{self, Window};
use winit;

// TODO: This value is just copied from an example, need to:
// 1. Verify that this is actually a good default
// 2. Allow for choosing a custom depth format
// 3. Validate the format (whether default or custom selected)
const DEPTH_FORMAT: Format = Format::D16Unorm;

/// The user function type for initialising their model.
pub type ModelFn<Model> = fn(&App) -> Model;

/// The user function type for updating their model in accordance with some event.
pub type EventFn<Model, Event> = fn(&App, &mut Model, Event);

/// The user function type for updating the user model within the application loop.
pub type UpdateFn<Model> = fn(&App, &mut Model, Update);

/// The user function type for drawing their model to the surface of a single window.
pub type ViewFn<Model> = fn(&App, &Model, Frame) -> Frame;

/// A shorthand version of `ViewFn` for sketches where the user does not need a model.
pub type SketchViewFn = fn(&App, Frame) -> Frame;

/// The user function type allowing them to consume the `model` when the application exits.
pub type ExitFn<Model> = fn(&App, Model);

/// The **App**'s view function.
enum View<Model = ()> {
    /// A view function allows for viewing the user's model.
    WithModel(ViewFn<Model>),
    /// A **Simple** view function does not require a user **Model**. Simpler to get started.
    Sketch(SketchViewFn),
}

/// A nannou `App` builder.
pub struct Builder<M = (), E = Event> {
    model: ModelFn<M>,
    event: Option<EventFn<M, E>>,
    update: Option<UpdateFn<M>>,
    default_view: Option<View<M>>,
    exit: Option<ExitFn<M>>,
    vulkan_instance: Option<Arc<vulkano::instance::Instance>>,
    vulkan_debug_callback: Option<gpu::VulkanDebugCallbackBuilder>,
    create_default_window: bool,
}

/// The default `model` function used when none is specified by the user.
fn default_model(_: &App) -> () {
    ()
}

/// Each nannou application has a single **App** instance. This **App** represents the entire
/// context of the application.
///
/// The **App** provides access to most "IO" related APIs. In other words, if you need access to
/// windowing, audio devices, laser fixtures, etc, the **App** will provide access to this.
///
/// The **App** owns and manages:
///
/// - The **window and input event loop** used to drive the application forward.
/// - **All windows** for graphics and user input. Windows can be referenced via their IDs.
/// - The **audio event loop** from which you can receive or send audio via streams.
pub struct App {
    config: RefCell<Config>,
    pub(crate) vulkan_instance: Arc<vulkano::instance::Instance>,
    pub(crate) events_loop: winit::EventsLoop,
    pub(crate) windows: RefCell<HashMap<window::Id, Window>>,
    draw_state: DrawState,
    pub(crate) ui: ui::Arrangement,
    /// The window that is currently in focus.
    pub(crate) focused_window: RefCell<Option<window::Id>>,

    /// Indicates whether or not the events loop is currently asleep.
    ///
    /// This is set to `true` each time the events loop is ready to return and the `LoopMode` is
    /// set to `Wait` for events.
    ///
    /// This value is set back to `false` each time the events loop receives any kind of event.
    pub(crate) events_loop_is_asleep: Arc<AtomicBool>,

    /// The `App`'s audio-related API.
    pub audio: Audio,

    /// The current state of the `Mouse`.
    pub mouse: state::Mouse,
    /// State of the keyboard keys.
    ///
    /// `mods` provides state of each of the modifier keys: `shift`, `ctrl`, `alt`, `logo`.
    ///
    /// `down` is the set of keys that are currently pressed.
    ///
    /// NOTE: `down` this is tracked by the nannou `App` so issues might occur if e.g. a key is
    /// pressed while the app is in focus and then released when out of focus. Eventually we should
    /// change this to query the OS somehow, but I don't think `winit` provides a way to do this
    /// yet.
    pub keys: state::Keys,
    /// Key time measurements tracked by the App.
    ///
    /// `duration.since_start` specifies the duration since the app started running.
    ///
    /// `duration.since_prev_update` specifies the duration since the previous update event.
    pub duration: state::Time,

    /// The time in seconds since the `App` started running.
    ///
    /// Primarily, this field is a convenience that removes the need to call
    /// `app.duration.since_start.secs()`. Normally we would try to avoid using such an ambiguous
    /// field name, however due to the sheer amount of use that this value has we feel it is
    /// beneficial to provide easier access.
    ///
    /// This value is of the same type as the scalar value used for describing space in animations.
    /// This makes it very easy to animate graphics and create changes over time without having to
    /// cast values or repeatedly calculate it from a `Duration` type. A small example might be
    /// `app.time.sin()` for simple oscillation behaviour.
    ///
    /// **Note:** This is suitable for use in short sketches, however should be avoided in long
    /// running installations. This is because the "resolution" of floating point values reduces as
    /// the number becomes higher. Instead, we recommend using `app.duration.since_start` or
    /// `app.duration.since_prev_update` to access a more precise form of app time.
    pub time: DrawScalar,
}

// /// Graphics related items within the `App`.
// pub struct Graphics {
// }
//
// /// Items related to the presence of one or more windows within the App.
// pub struct Windowing {
// }

/// Miscellaneous app configuration parameters.
#[derive(Debug)]
struct Config {
    loop_mode: LoopMode,
    exit_on_escape: bool,
    fullscreen_on_shortcut: bool,
}

/// A **nannou::Draw** instance owned by the **App**. A simple API for sketching with 2D and 3D
/// graphics.
///
/// This is a conveniently accessible **Draw** instance which can be easily re-used between calls
/// to an app's **view** function.
#[derive(Debug)]
pub struct Draw<'a> {
    window_id: window::Id,
    draw: RefMut<'a, draw::Draw<DrawScalar>>,
    renderer: RefMut<'a, RefCell<draw::backend::vulkano::Renderer>>,
}

// Draw state managed by the **App**.
#[derive(Debug)]
struct DrawState {
    draw: RefCell<draw::Draw<DrawScalar>>,
    renderer: RefCell<Option<RefCell<draw::backend::vulkano::Renderer>>>,
}

/// The app uses a set scalar type in order to provide a simplistic API to users.
///
/// If you require changing the scalar type to something else, consider using a custom
/// **nannou::draw::Draw** instance.
pub type DrawScalar = geom::scalar::Default;

/// An API accessed via `app.audio` for enumerating audio devices and spawning input/output audio
/// streams with either default or custom stream format.
pub struct Audio {
    event_loop: Arc<cpal::EventLoop>,
    process_fn_tx: RefCell<Option<mpsc::Sender<audio::stream::ProcessFnMsg>>>,
}

/// A handle to the **App** that can be shared across threads. This may be used to "wake up" the
/// **App**'s inner event loop.
pub struct Proxy {
    events_loop_proxy: winit::EventsLoopProxy,
    events_loop_is_asleep: Arc<AtomicBool>,
}

// The type returned by a mode-specific run loop within the `run_loop` function.
struct Break<M> {
    model: M,
    reason: BreakReason,
}

// The reason why the mode-specific run loop broke out of looping.
enum BreakReason {
    // The application has exited.
    Exit,
    // The LoopMode has been changed to the given mode.
    NewLoopMode(LoopMode),
}

// State related specifically to the application loop, shared between loop modes.
struct LoopContext<M, E> {
    // The user's application event function.
    event_fn: Option<EventFn<M, E>>,
    // The user's update function.
    update_fn: Option<UpdateFn<M>>,
    // The user's default function for drawing to a window's swapchain's image, used in the case
    // that the user has not provided a window-specific view function.
    default_view: Option<View<M>>,
    // The moment at which `run_loop` began.
    loop_start: Instant,
    // A buffer for collecting polled events.
    winit_events: Vec<winit::Event>,
    // The last instant that `update` was called. Initialised to `loop_start`.
    last_update: Instant,
    // The instant that the loop last ended. Initialised to `loop_start`.
    last_loop_end: Instant,
    // The number of updates yet to be called (for `LoopMode::Wait`).
    updates_remaining: usize,
}

/// The mode in which the **App** is currently running the event loop and emitting `Update` events.
#[derive(Clone, Debug, PartialEq)]
pub enum LoopMode {
    /// Specifies that the application is continuously looping at a consistent rate.
    ///
    /// An application running in the **Rate** loop mode will behave as follows:
    ///
    /// 1. Poll for and collect all pending user input. `event` is then called with all application
    ///    events that have occurred.
    ///
    /// 2. `event` is called with an `Update` event.
    ///
    /// 3. Check the time and sleep for the remainder of the `update_interval` then go to 1.
    ///
    /// `view` is called at an arbitraty rate by the vulkan swapchain for each window. It uses
    /// whatever the state of the user's model happens to be at that moment in time.
    Rate {
        /// The minimum interval between emitted updates.
        update_interval: Duration,
    },

    /// Waits for user input events to occur before calling `event` with an `Update` event.
    ///
    /// This is particularly useful for low-energy GUIs that only need to update when some sort of
    /// input has occurred. The benefit of using this mode is that you don't waste CPU cycles
    /// looping or updating when you know nothing is changing in your model or view.
    Wait {
        /// The number of `update`s (and in turn `view` calls per window) that should occur since
        /// the application last received a non-`Update` event.
        updates_following_event: usize,
        /// The minimum interval between emitted updates.
        update_interval: Duration,
    },

    /// Synchronises `Update` events with requests for a new image by the swapchain for each
    /// window in order to achieve minimal latency between the state of the model and what is
    /// displayed on screen. This mode should be particularly useful for interactive applications
    /// and games where minimal latency between user input and the display image is essential.
    ///
    /// The result of using this loop mode is similar to using vsync in traditional applications.
    /// E.g. if you have one window running on a monitor with a 60hz refresh rate, your update will
    /// get called at a fairly consistent interval that is close to 60 times per second.
    ///
    /// It is worth noting that, in the case that you have more than one window and they are
    /// situated on different displays with different refresh rates, `update` will almost certainly
    /// not be called at a consistent interval. Instead, it will be called as often as necessary -
    /// if it has been longer than `minimum_update_interval` or if some user input was received
    /// since the last `Update`. That said, each `Update` event contains the duration since the
    /// last `Update` occurred, so as long as all time-based state (like animations or physics
    /// simulations) are driven by this, the `update` interval consistency should not cause issues.
    ///
    /// ### The Swapchain
    ///
    /// The purpose of the swapchain for each window is to synchronise the presentation of images
    /// (calls to `view` in nannou) with the refresh rate of the screen. *You can learn more about
    /// the swap chain
    /// [here](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain).*
    RefreshSync {
        /// The minimum duration that must occur between calls to `update`. Under the `RefreshSync`
        /// mode, the application loop will attempt to emit an `Update` event once every time a new
        /// image is acquired from a window's swapchain. Thus, this value is very useful when
        /// working with multiple windows in order to avoid updating at an unnecessarily high rate.
        ///
        /// We recommend using a `Duration` that is roughly half the duration between refreshes of
        /// the window on the display with the highest refresh rate. For example, if the highest
        /// display refresh rate is 60hz (with an interval of ~16ms) a suitable
        /// `minimum_update_interval` might be 8ms. This should result in `update` being called
        /// once every 16ms regardless of the number of windows.
        minimum_update_interval: Duration,
        /// The windows to which `Update` events should be synchronised.
        ///
        /// If this is `Some`, an `Update` will only occur for those windows that are contained
        /// within this set. This is particularly useful if you only want to synchronise your
        /// updates with one or more "main" windows and you don't mind so much about the latency
        /// for the rest.
        ///
        /// If this is `None` (the default case), `Update` events will be synchronised with *all*
        /// windows.
        windows: Option<HashSet<window::Id>>,
    },
}

impl<M> Builder<M, Event>
where
    M: 'static,
{
    /// Begin building the `App`.
    ///
    /// The `model` argument is the function that the App will call to initialise your Model.
    ///
    /// The Model can be thought of as the state that you would like to track throughout the
    /// lifetime of your nannou program from start to exit.
    ///
    /// The given function is called before any event processing begins within the application.
    ///
    /// The Model that is returned by the function is the same model that will be passed to the
    /// given event and view functions.
    pub fn new(model: ModelFn<M>) -> Self {
        Builder {
            model,
            event: None,
            update: None,
            default_view: None,
            exit: None,
            vulkan_instance: None,
            vulkan_debug_callback: None,
            create_default_window: false,
        }
    }

    /// The function that the app will call to allow you to update your Model on events.
    ///
    /// The `event` function allows you to expect any event type that implements `LoopEvent`,
    /// however nannou also provides a default `Event` type that should cover most use cases. This
    /// event type is an `enum` that describes all the different kinds of I/O events that might
    /// occur during the life of the program. These include things like `Update`s and
    /// `WindowEvent`s such as `KeyPressed`, `MouseMoved`, and so on.
    pub fn event<E>(self, event: EventFn<M, E>) -> Builder<M, E>
    where
        E: LoopEvent,
    {
        let Builder {
            model,
            update,
            default_view,
            exit,
            create_default_window,
            vulkan_instance,
            vulkan_debug_callback,
            ..
        } = self;
        Builder {
            model,
            event: Some(event),
            update,
            default_view,
            exit,
            create_default_window,
            vulkan_instance,
            vulkan_debug_callback,
        }
    }
}

impl<M, E> Builder<M, E>
where
    M: 'static,
    E: LoopEvent,
{
    /// The default `view` function that the app will call to allow you to present your Model to
    /// the surface of a window on your display.
    ///
    /// This function will be used in the case that a window-specific view function has not been
    /// provided, e.g. via `window::Builder::view` or `window::Builder::sketch`.
    ///
    /// Note that when working with more than one window, you can use `frame.window_id()` to
    /// determine which window the current call is associated with.
    pub fn view(mut self, view: ViewFn<M>) -> Self {
        self.default_view = Some(View::WithModel(view));
        self
    }

    /// A function for updating the model within the application loop.
    ///
    /// See the `LoopMode` documentation for more information about the different kinds of
    /// application loop modes available in nannou and how they behave.
    ///
    /// Update events are also emitted as a variant of the `event` function. Note that if you
    /// specify both an `event` function and an `update` function, the `event` function will always
    /// be called with an update event prior to this `update` function.
    pub fn update(mut self, update: UpdateFn<M>) -> Self {
        self.update = Some(update);
        self
    }

    /// Tell the app that you would like it to create a single, simple, default window just before
    /// it calls your model function.
    ///
    /// The given `view` function will play the same role as if passed to the `view` builder
    /// method. Note that the `view` function passed to this method will overwrite any pre-existing
    /// view function specified by any preceding call to the `view`
    ///
    /// Note that calling this multiple times will not give you multiple windows, but instead will
    /// simply overwrite pre-existing calls to the method. If you would like to create multiple
    /// windows or would like more flexibility in your window creation process, please see the
    /// `App::new_window` method. The role of this `simple_window` method is to provide a
    /// quick-and-easy way to start with a simple window. This can be very useful for quick ideas,
    /// small single-window applications and examples.
    pub fn simple_window(mut self, view: ViewFn<M>) -> Self {
        self.default_view = Some(View::WithModel(view));
        self.create_default_window = true;
        self
    }

    /// Specify an `exit` function to be called when the application exits.
    ///
    /// The exit function gives ownership of the model back to you for any cleanup that might be
    /// necessary.
    pub fn exit(mut self, exit: ExitFn<M>) -> Self {
        self.exit = Some(exit);
        self
    }

    /// The vulkan instance to use for interfacing with the system vulkan API.
    ///
    /// If unspecified, nannou will create one via the following:
    ///
    /// ```norun
    /// # extern crate nannou;
    /// # fn main() {
    /// nannou::gpu::VulkanInstanceBuilder::new()
    ///     .build()
    ///     .expect("failed to creat vulkan instance")
    /// # ;
    /// # }
    /// ```
    ///
    /// If a `vulkan_debug_callback` was specified but the `vulkan_instance` is unspecified, nannou
    /// will do the following:
    ///
    /// ```norun
    /// # extern crate nannou;
    /// # fn main() {
    /// nannou::gpu::VulkanInstanceBuilder::new()
    ///     .extensions(nannou::vulkano::instance::InstanceExtensions {
    ///         ext_debug_report: true,
    ///         ..nannou::gpu::required_windowing_extensions()
    ///     })
    ///     .layers(vec!["VK_LAYER_LUNARG_standard_validation"])
    ///     .build()
    ///     .expect("failed to creat vulkan instance")
    /// # ;
    /// # }
    /// ```
    pub fn vulkan_instance(mut self, vulkan_instance: Arc<vulkano::instance::Instance>) -> Self {
        self.vulkan_instance = Some(vulkan_instance);
        self
    }

    /// Specify a debug callback to be used with the vulkan instance.
    ///
    /// If you just want to print messages from the standard validation layers to stdout, you can
    /// call this method with `Default::default()` as the argument.
    ///
    /// Note that if you have specified a custom `vulkan_instance`, that instance must have the
    /// `ext_debug_report` extension enabled and must have been constructed with a debug layer.
    pub fn vulkan_debug_callback(mut self, debug_cb: gpu::VulkanDebugCallbackBuilder) -> Self {
        self.vulkan_debug_callback = Some(debug_cb);
        self
    }

    /// Build and run an `App` with the specified parameters.
    ///
    /// This function will not return until the application has exited.
    ///
    /// If you wish to remain cross-platform frienly, we recommend that you call this on the main
    /// thread as some platforms require that their application event loop and windows are
    /// initialised on the main thread.
    pub fn run(mut self) {
        // Start the winit window event loop.
        let events_loop = winit::EventsLoop::new();

        // Keep track of whether or not a debug cb was specified so we know what default extensions
        // and layers are necessary.
        let debug_callback_specified = self.vulkan_debug_callback.is_some();

        // The vulkan instance necessary for graphics.
        let vulkan_instance = self.vulkan_instance.take().unwrap_or_else(|| {
            if debug_callback_specified {
                gpu::VulkanInstanceBuilder::new()
                    .extensions(InstanceExtensions {
                        ext_debug_report: true,
                        ..gpu::required_windowing_extensions()
                    })
                    .layers(vec!["VK_LAYER_LUNARG_standard_validation"])
                    .build()
                    .expect("failed to create vulkan instance")
            } else {
                gpu::VulkanInstanceBuilder::new()
                    .build()
                    .expect("failed to create vulkan instance")
            }
        });

        // If a callback was specified, build it with the created instance.
        let _vulkan_debug_callback = self.vulkan_debug_callback
            .take()
            .map(|builder| {
                builder
                    .build(&vulkan_instance)
                    .expect("failed to build vulkan debug callback")
            });

        // Initialise the app.
        let app = App::new(events_loop, vulkan_instance).expect("failed to construct `App`");

        // Create the default window if necessary
        if self.create_default_window {
            let window_id = app
                .new_window()
                .build()
                .expect("could not build default app window");
            *app.focused_window.borrow_mut() = Some(window_id);
        }

        // Call the user's model function.
        let model = (self.model)(&app);

        // If there is not yet some default window in "focus" check to see if one has been created.
        if app.focused_window.borrow().is_none() {
            if let Some(id) = app.windows.borrow().keys().next() {
                *app.focused_window.borrow_mut() = Some(id.clone());
            }
        }

        // If the loop mode was set in the user's model function, ensure all window swapchains are
        // re-created appropriately.
        let loop_mode = app.loop_mode();
        if loop_mode != LoopMode::default() {
            let mut windows = app.windows.borrow_mut();
            for window in windows.values_mut() {
                let min_image_count = window
                    .surface
                    .capabilities(window.swapchain.device().physical_device())
                    .expect("failed to get surface capabilities")
                    .min_image_count;
                let user_specified_present_mode = window.user_specified_present_mode;
                let user_specified_image_count = window.user_specified_image_count;
                let (present_mode, image_count) = window::preferred_present_mode_and_image_count(
                    &loop_mode,
                    min_image_count,
                    user_specified_present_mode,
                    user_specified_image_count,
                );
                if window.swapchain.present_mode() != present_mode
                || window.swapchain.num_images() != image_count {
                    change_loop_mode_for_window(window, &loop_mode);
                }
            }
        }

        run_loop(app, model, self.event, self.update, self.default_view, self.exit);
    }
}

impl Builder<(), Event> {
    /// Shorthand for building a simple app that has no model, handles no events and simply draws
    /// to a single window.
    ///
    /// This is useful for late night hack sessions where you just don't care about all that other
    /// stuff, you just want to play around with some ideas or make something pretty.
    pub fn sketch(view: SketchViewFn) {
        let builder: Self = Builder {
            model: default_model,
            event: None,
            update: None,
            default_view: Some(View::Sketch(view)),
            exit: None,
            create_default_window: true,
            vulkan_instance: None,
            vulkan_debug_callback: None,
        };
        builder.run()
    }
}

/// Given some "frames per second", return the interval between frames as a `Duration`.
fn update_interval(fps: f64) -> Duration {
    assert!(fps > 0.0);
    const NANOSEC_PER_SEC: f64 = 1_000_000_000.0;
    let interval_nanosecs = NANOSEC_PER_SEC / fps;
    let secs = (interval_nanosecs / NANOSEC_PER_SEC) as u64;
    let nanosecs = (interval_nanosecs % NANOSEC_PER_SEC) as u32;
    Duration::new(secs, nanosecs)
}

impl LoopMode {
    pub const DEFAULT_RATE_FPS: f64 = 60.0;
    pub const DEFAULT_UPDATES_FOLLOWING_EVENT: usize = 3;

    /// Specify the **Rate** mode with the given frames-per-second.
    pub fn rate_fps(fps: f64) -> Self {
        let update_interval = update_interval(fps);
        LoopMode::Rate { update_interval }
    }

    /// Specify the **Wait** mode with the given number of updates following each non-`Update`
    /// event.
    ///
    /// Uses the default update interval.
    pub fn wait(updates_following_event: usize) -> Self {
        let update_interval = update_interval(Self::DEFAULT_RATE_FPS);
        LoopMode::Wait {
            updates_following_event,
            update_interval,
        }
    }

    /// Specify the **Wait** mode with the given number of updates following each non-`Update`
    /// event.
    ///
    /// Waits long enough to ensure loop iteration never occurs faster than the given `max_fps`.
    pub fn wait_with_max_fps(updates_following_event: usize, max_fps: f64) -> Self {
        let update_interval = update_interval(max_fps);
        LoopMode::Wait {
            updates_following_event,
            update_interval,
        }
    }

    /// Specify the **Wait** mode with the given number of updates following each non-`Update`
    /// event.
    ///
    /// Waits long enough to ensure loop iteration never occurs faster than the given `max_fps`.
    pub fn wait_with_interval(updates_following_event: usize, update_interval: Duration) -> Self {
        LoopMode::Wait {
            updates_following_event,
            update_interval,
        }
    }

    /// A simplified constructor for the default `RefreshSync` loop mode.
    ///
    /// Assumes a display refresh rate of ~60hz and in turn specifies a `minimum_update_latency` of
    /// ~8.33ms. The `windows` field is set to `None`.
    pub fn refresh_sync() -> Self {
        LoopMode::RefreshSync {
            minimum_update_interval: update_interval(Self::DEFAULT_RATE_FPS * 2.0),
            windows: None,
        }
    }
}

impl Default for LoopMode {
    fn default() -> Self {
        LoopMode::refresh_sync()
    }
}

impl Default for Config {
    fn default() -> Self {
        let loop_mode = Default::default();
        let exit_on_escape = App::DEFAULT_EXIT_ON_ESCAPE;
        let fullscreen_on_shortcut = App::DEFAULT_FULLSCREEN_ON_SHORTCUT;
        Config {
            loop_mode,
            exit_on_escape,
            fullscreen_on_shortcut,
        }
    }
}

impl App {
    pub const ASSETS_DIRECTORY_NAME: &'static str = "assets";
    pub const DEFAULT_EXIT_ON_ESCAPE: bool = true;
    pub const DEFAULT_FULLSCREEN_ON_SHORTCUT: bool = true;

    // Create a new `App`.
    pub(super) fn new(
        events_loop: winit::EventsLoop,
        vulkan_instance: Arc<vulkano::instance::Instance>,
    ) -> Result<Self, vulkano::instance::InstanceCreationError> {
        let windows = RefCell::new(HashMap::new());
        let draw = RefCell::new(draw::Draw::default());
        let config = RefCell::new(Default::default());
        let renderer = RefCell::new(None);
        let draw_state = DrawState { draw, renderer };
        let cpal_event_loop = Arc::new(cpal::EventLoop::new());
        let process_fn_tx = RefCell::new(None);
        let audio = Audio {
            event_loop: cpal_event_loop,
            process_fn_tx,
        };
        let focused_window = RefCell::new(None);
        let ui = ui::Arrangement::new();
        let mouse = state::Mouse::new();
        let keys = state::Keys::default();
        let duration = state::Time::default();
        let time = duration.since_start.secs() as _;
        let events_loop_is_asleep = Arc::new(AtomicBool::new(false));
        let app = App {
            vulkan_instance,
            events_loop,
            events_loop_is_asleep,
            focused_window,
            windows,
            config,
            draw_state,
            audio,
            ui,
            mouse,
            keys,
            duration,
            time,
        };
        Ok(app)
    }

    /// A reference to the vulkan instance associated with the `App`.
    ///
    /// If you would like to construct the app with a custom vulkan instance, see the
    /// `app::Builder::vulkan_instance` method.
    pub fn vulkan_instance(&self) -> &Arc<vulkano::instance::Instance> {
        &self.vulkan_instance
    }

    /// Returns an iterator yielding each of the physical devices on the system that are vulkan
    /// compatible.
    ///
    /// If a physical device is not specified for a window surface's swapchain, the first device
    /// yielded by this iterator is used as the default.
    pub fn vulkan_physical_devices(&self) -> vulkano::instance::PhysicalDevicesIter {
        vulkano::instance::PhysicalDevice::enumerate(&self.vulkan_instance)
    }

    /// Retrieve the default vulkan physical device.
    ///
    /// This is simply the first device yielded by the `vulkan_physical_devices` method.
    pub fn default_vulkan_physical_device(&self) -> Option<vulkano::instance::PhysicalDevice> {
        self.vulkan_physical_devices().next()
    }

    /// Find and return the absolute path to the project's `assets` directory.
    ///
    /// This method looks for the assets directory in the following order:
    ///
    /// 1. Checks the same directory as the executable.
    /// 2. Recursively checks exe's parent directories (to a max depth of 5).
    /// 3. Recursively checks exe's children directories (to a max depth of 3).
    pub fn assets_path(&self) -> Result<PathBuf, find_folder::Error> {
        let exe_path = std::env::current_exe()?;
        find_folder::Search::ParentsThenKids(5, 3)
            .of(exe_path
                .parent()
                .expect("executable has no parent directory to search")
                .into())
            .for_folder(Self::ASSETS_DIRECTORY_NAME)
    }

    /// Begin building a new OpenGL window.
    pub fn new_window(&self) -> window::Builder {
        window::Builder::new(self)
    }

    /// The number of windows currently in the application.
    pub fn window_count(&self) -> usize {
        self.windows.borrow().len()
    }

    /// A reference to the window with the given `Id`.
    pub fn window(&self, id: window::Id) -> Option<std::cell::Ref<Window>> {
        let windows = self.windows.borrow();
        if !windows.contains_key(&id) {
            None
        } else {
            Some(std::cell::Ref::map(windows, |ws| &ws[&id]))
        }
    }

    /// Return the **Id** of the currently focused window.
    ///
    /// **Panics** if there are no windows or if no window is in focus.
    pub fn window_id(&self) -> window::Id {
        self.focused_window
            .borrow()
            .expect("called `App::window_id` but there is no window currently in focus")
    }

    /// Return a `Vec` containing a unique `window::Id` for each currently open window managed by
    /// the `App`.
    pub fn window_ids(&self) -> Vec<window::Id> {
        let windows = self.windows.borrow();
        windows.keys().cloned().collect()
    }

    /// Return the **Rect** for the currently focused window.
    ///
    /// The **Rect** coords are described in "points" (pixels divided by the hidpi factor).
    ///
    /// **Panics** if there are no windows or if no window is in focus.
    pub fn window_rect(&self) -> geom::Rect<DrawScalar> {
        let (w, h) = self.main_window().inner_size_points();
        geom::Rect::from_w_h(w as _, h as _)
    }

    /// A reference to the window currently in focus.
    ///
    /// **Panics** if their are no windows open in the **App**.
    ///
    /// Uses the **App::window** method internally.
    ///
    /// TODO: Currently this produces a reference to the *focused* window, but this behaviour
    /// should be changed to track the "main" window (the first window created?).
    pub fn main_window(&self) -> std::cell::Ref<Window> {
        self.window(self.window_id())
            .expect("no window for focused id")
    }

    /// Return whether or not the `App` is currently set to exit when the `Escape` key is pressed.
    pub fn exit_on_escape(&self) -> bool {
        self.config.borrow().exit_on_escape
    }

    /// Specify whether or not the app should close when the `Escape` key is pressed.
    ///
    /// By default this is `true`.
    pub fn set_exit_on_escape(&self, b: bool) {
        self.config.borrow_mut().exit_on_escape = b;
    }

    /// Returns whether or not the `App` is currently allows the focused window to enter or exit
    /// fullscreen via typical platform-specific shortcuts.
    ///
    /// - Linux uses F11.
    /// - macOS uses apple key + f.
    /// - Windows uses windows key + f.
    pub fn fullscreen_on_shortcut(&self) -> bool {
        self.config.borrow().fullscreen_on_shortcut
    }

    /// Set whether or not the `App` should allow the focused window to enter or exit fullscreen
    /// via typical platform-specific shortcuts.
    ///
    /// - Linux uses F11.
    /// - macOS uses apple key + f.
    /// - Windows uses windows key + f.
    pub fn set_fullscreen_on_shortcut(&self, b: bool) {
        self.config.borrow_mut().fullscreen_on_shortcut = b;
    }

    /// Returns the **App**'s current **LoopMode**.
    ///
    /// The default loop mode is `Rate` at 60 frames per second (an `update_interval` of ~16ms).
    pub fn loop_mode(&self) -> LoopMode {
        self.config.borrow().loop_mode.clone()
    }

    /// Sets the loop mode of the **App**.
    ///
    /// Note: Setting the loop mode will not affect anything until the end of the current loop
    /// iteration. The behaviour of a single loop iteration is described under each of the
    /// **LoopMode** variants.
    pub fn set_loop_mode(&self, mode: LoopMode) {
        self.config.borrow_mut().loop_mode = mode;
    }

    /// A handle to the **App** that can be shared across threads.
    ///
    /// This can be used to "wake up" the **App**'s inner event loop.
    pub fn create_proxy(&self) -> Proxy {
        let events_loop_proxy = self.events_loop.create_proxy();
        let events_loop_is_asleep = self.events_loop_is_asleep.clone();
        Proxy {
            events_loop_proxy,
            events_loop_is_asleep,
        }
    }

    /// A builder for creating a new **Ui**.
    ///
    /// Each **Ui** is associated with one specific window. By default, this is the window returned
    /// by `App::window_id` (the currently focused window).
    pub fn new_ui(&self) -> ui::Builder {
        ui::Builder::new(self)
    }

    /// Produce the **App**'s **Draw** API for drawing geometry and text with colors and textures.
    ///
    /// **Note:** There may only be a single **app::Draw** instance at any point in time. If this
    /// method is called while there is a pre-existing instance of **app::Draw** this method will
    /// **panic**.
    ///
    /// Returns **None** if there is no window for the given **window::Id**.
    pub fn draw_for_window(&self, window_id: window::Id) -> Option<Draw> {
        let window = match self.window(window_id) {
            None => return None,
            Some(window) => window,
        };
        let device = window.swapchain.swapchain.device().clone();
        let color_format = window.swapchain.swapchain.format();
        let draw = self.draw_state.draw.borrow_mut();
        draw.reset();
        if self.draw_state.renderer.borrow().is_none() {
            let renderer = draw::backend::vulkano::Renderer::new(device, color_format, DEPTH_FORMAT)
                .expect("failed to create `Draw` renderer for vulkano backend");
            *self.draw_state.renderer.borrow_mut() = Some(RefCell::new(renderer));
        }
        let renderer = self.draw_state.renderer.borrow_mut();
        let renderer = RefMut::map(renderer, |r| r.as_mut().unwrap());
        Some(Draw {
            window_id,
            draw,
            renderer,
        })
    }

    /// Produce the **App**'s **Draw** API for drawing geometry and text with colors and textures.
    ///
    /// This is a simplified wrapper around the **App::draw_for_window** method that draws to the
    /// window currently in focus.
    ///
    /// **Panics** if there are no windows open.
    ///
    /// **Note:** There may only be a single **app::Draw** instance at any point in time. If this
    /// method is called while there is a pre-existing instance of **app::Draw** this method will
    /// **panic**.
    pub fn draw(&self) -> Draw {
        self.draw_for_window(self.window_id())
            .expect("no window open for `app.window_id`")
    }

    /// The number of times the focused window's **view** function has been called since the start
    /// of the program.
    pub fn elapsed_frames(&self) -> u64 {
        self.main_window().frame_count
    }
}

impl Audio {
    /// Enumerate the available audio devices on the system.
    ///
    /// Produces an iterator yielding `audio::Device`s.
    pub fn devices(&self) -> audio::Devices {
        let devices = cpal::devices();
        audio::Devices { devices }
    }

    /// Enumerate the available audio devices on the system that support input streams.
    ///
    /// Produces an iterator yielding `audio::Device`s.
    pub fn input_devices(&self) -> audio::stream::input::Devices {
        let devices = cpal::input_devices();
        audio::stream::input::Devices { devices }
    }

    /// Enumerate the available audio devices on the system that support output streams.
    ///
    /// Produces an iterator yielding `audio::Device`s.
    pub fn output_devices(&self) -> audio::stream::output::Devices {
        let devices = cpal::output_devices();
        audio::stream::output::Devices { devices }
    }

    /// The current default audio input device.
    pub fn default_input_device(&self) -> Option<audio::Device> {
        cpal::default_input_device().map(|device| audio::Device { device })
    }

    /// The current default audio output device.
    pub fn default_output_device(&self) -> Option<audio::Device> {
        cpal::default_output_device().map(|device| audio::Device { device })
    }

    /// Begin building a new input audio stream.
    ///
    /// If this is the first time a stream has been created, this method will spawn the
    /// `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    pub fn new_input_stream<M, F, S>(
        &self,
        model: M,
        capture: F,
    ) -> audio::stream::input::Builder<M, F, S> {
        audio::stream::input::Builder {
            capture,
            builder: self.new_stream(model),
        }
    }

    /// Begin building a new output audio stream.
    ///
    /// If this is the first time a stream has been created, this method will spawn the
    /// `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    pub fn new_output_stream<M, F, S>(
        &self,
        model: M,
        render: F,
    ) -> audio::stream::output::Builder<M, F, S> {
        audio::stream::output::Builder {
            render,
            builder: self.new_stream(model),
        }
    }

    // Builder initialisation shared between input and output streams.
    //
    // If this is the first time a stream has been created, this method will spawn the
    // `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    fn new_stream<M, S>(&self, model: M) -> audio::stream::Builder<M, S> {
        let process_fn_tx = if self.process_fn_tx.borrow().is_none() {
            let event_loop = self.event_loop.clone();
            let (tx, rx) = mpsc::channel();
            let mut loop_context = audio::stream::LoopContext::new(rx);
            thread::Builder::new()
                .name("cpal::EventLoop::run thread".into())
                .spawn(move || event_loop.run(move |id, data| loop_context.process(id, data)))
                .expect("failed to spawn cpal::EventLoop::run thread");
            *self.process_fn_tx.borrow_mut() = Some(tx.clone());
            tx
        } else {
            self.process_fn_tx.borrow().as_ref().unwrap().clone()
        };

        audio::stream::Builder {
            event_loop: self.event_loop.clone(),
            process_fn_tx: process_fn_tx,
            model,
            sample_rate: None,
            channels: None,
            frames_per_buffer: None,
            device: None,
            sample_format: PhantomData,
        }
    }
}

impl Proxy {
    /// Wake up the application!
    ///
    /// This wakes up the **App**'s inner event loop and inserts an **Awakened** event.
    ///
    /// The `app::Proxy` stores a flag in order to track whether or not the `EventsLoop` is
    /// currently blocking and waiting for events. This method will only call the underlying
    /// `winit::EventsLoopProxy::wakeup` method if this flag is set to true and will immediately
    /// set the flag to false afterwards. This makes it safe to call the `wakeup` method as
    /// frequently as necessary across methods without causing any underlying OS methods to be
    /// called more than necessary.
    pub fn wakeup(&self) -> Result<(), winit::EventsLoopClosed> {
        if self.events_loop_is_asleep.load(atomic::Ordering::Relaxed) {
            self.events_loop_proxy.wakeup()?;
            self.events_loop_is_asleep
                .store(false, atomic::Ordering::Relaxed);
        }
        Ok(())
    }
}

impl<'a> Draw<'a> {
    /// Draw the current state of the inner mesh to the given frame.
    pub fn to_frame(
        &self,
        app: &App,
        frame: &Frame,
    ) -> Result<(), draw::backend::vulkano::DrawError> {
        let window = app
            .window(self.window_id)
            .expect("no window to draw to for `app::Draw`'s window_id");
        assert_eq!(
            self.window_id,
            frame.window_id(),
            "attempted to draw content intended for window {:?} in a frame \
            associated with window {:?}",
            self.window_id,
            frame.window_id(),
        );
        let dpi_factor = window.hidpi_factor();
        let mut renderer = self.renderer.borrow_mut();
        renderer.draw_to_frame(&self.draw, dpi_factor, frame, DEPTH_FORMAT)
    }
}

impl<'a> Deref for Draw<'a> {
    type Target = RefMut<'a, draw::Draw<DrawScalar>>;
    fn deref(&self) -> &Self::Target {
        &self.draw
    }
}

impl<'a> DerefMut for Draw<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.draw
    }
}

// Application Loop.
//
// Beyond this point lies the master function for running the main application loop!
//
// This is undoubtedly the hairiest part of nannou's code base. This is largely due to the fact
// that it is the part of nannou where we marry application and user input events, loop timing,
// updating the model, platform-specific quirks and warts, the various possible `LoopMode`s and
// Vulkan interop.
//
// If you would like to contribute but are unsure about any of the following, feel free to open an
// issue and ask!
fn run_loop<M, E>(
    mut app: App,
    mut model: M,
    event_fn: Option<EventFn<M, E>>,
    update_fn: Option<UpdateFn<M>>,
    default_view: Option<View<M>>,
    exit_fn: Option<ExitFn<M>>,
)
where
    M: 'static,
    E: LoopEvent,
{
    let loop_start = Instant::now();

    // Initialise the loop context.
    let mut loop_ctxt = LoopContext {
        event_fn,
        update_fn,
        default_view,
        loop_start,
        winit_events: vec![],
        last_update: loop_start,
        last_loop_end: loop_start,
        updates_remaining: LoopMode::DEFAULT_UPDATES_FOLLOWING_EVENT,
    };

    let mut loop_mode = app.loop_mode();

    // Begin running the application loop based on the current `LoopMode`.
    'mode: loop {
        let Break { model: new_model, reason } = match loop_mode {
            LoopMode::Rate { update_interval } => {
                run_loop_mode_rate(
                    &mut app,
                    model,
                    &mut loop_ctxt,
                    update_interval,
                )
            }
            LoopMode::Wait { updates_following_event, update_interval, } => {
                loop_ctxt.updates_remaining = updates_following_event;
                run_loop_mode_wait(
                    &mut app,
                    model,
                    &mut loop_ctxt,
                    updates_following_event,
                    update_interval,
                )
            }
            LoopMode::RefreshSync { minimum_update_interval, windows } => {
                run_loop_mode_refresh_sync(
                    &mut app,
                    model,
                    &mut loop_ctxt,
                    minimum_update_interval,
                    windows,
                )
            }
        };

        model = new_model;
        match reason {
            // If the break reason was due to the `LoopMode` changing, switch to the new loop mode
            // and continue.
            BreakReason::NewLoopMode(new_loop_mode) => {
                loop_mode = new_loop_mode;
                change_loop_mode(&app, &loop_mode);
            },
            // If the loop broke due to the application exiting, we're done!
            BreakReason::Exit => {
                if let Some(exit_fn) = exit_fn {
                    exit_fn(&app, model);
                }
                return;
            }
        }
    }
}

// Run the application loop under the `Rate` mode.
fn run_loop_mode_rate<M, E>(
    app: &mut App,
    mut model: M,
    loop_ctxt: &mut LoopContext<M, E>,
    mut update_interval: Duration,
) -> Break<M>
where
    M: 'static,
    E: LoopEvent,
{
    loop {
        // Handle any pending window events.
        app.events_loop
            .poll_events(|event| loop_ctxt.winit_events.push(event));
        for winit_event in loop_ctxt.winit_events.drain(..) {
            let (new_model, exit) = process_and_emit_winit_event(
                app,
                model,
                loop_ctxt.event_fn,
                winit_event,
            );
            model = new_model;
            if exit {
                let reason = BreakReason::Exit;
                return Break { model, reason };
            }
        }

        // Update the app's durations.
        let now = Instant::now();
        let since_last = now.duration_since(loop_ctxt.last_update).into();
        let since_start = now.duration_since(loop_ctxt.loop_start).into();
        app.duration.since_start = since_start;
        app.duration.since_prev_update = since_last;
        app.time = app.duration.since_start.secs() as _;

        // Emit an update event.
        let update = update_event(loop_ctxt.loop_start, &mut loop_ctxt.last_update);
        if let Some(event_fn) = loop_ctxt.event_fn {
            let event = E::from(update.clone());
            event_fn(&app, &mut model, event);
        }
        if let Some(update_fn) = loop_ctxt.update_fn {
            update_fn(&app, &mut model, update);
        }

        // Draw to each window.
        for window_id in app.window_ids() {
            acquire_image_and_view_frame(app, window_id, &model, loop_ctxt.default_view.as_ref());
        }

        // Sleep if there's still some time left within the interval.
        let now = Instant::now();
        let since_last_loop_end = now.duration_since(loop_ctxt.last_loop_end);
        if since_last_loop_end < update_interval {
            std::thread::sleep(update_interval - since_last_loop_end);
        }
        loop_ctxt.last_loop_end = Instant::now();

        // See if the loop mode has changed. If so, break.
        update_interval = match app.loop_mode() {
            LoopMode::Rate { update_interval } => update_interval,
            loop_mode => {
                let reason = BreakReason::NewLoopMode(loop_mode);
                return Break { model, reason };
            }
        };
    }
}

// Run the application loop under the `Wait` mode.
fn run_loop_mode_wait<M, E>(
    app: &mut App,
    mut model: M,
    loop_ctxt: &mut LoopContext<M, E>,
    mut updates_following_event: usize,
    mut update_interval: Duration,
) -> Break<M>
where
    M: 'static,
    E: LoopEvent,
{
    loop {
        // First collect any pending window events.
        app.events_loop.poll_events(|event| loop_ctxt.winit_events.push(event));

        // If there are no events and the `Ui` does not need updating,
        // wait for the next event.
        if loop_ctxt.winit_events.is_empty() && loop_ctxt.updates_remaining == 0 {
            let events_loop_is_asleep = app.events_loop_is_asleep.clone();
            events_loop_is_asleep.store(true, atomic::Ordering::Relaxed);
            app.events_loop.run_forever(|event| {
                events_loop_is_asleep.store(false, atomic::Ordering::Relaxed);
                loop_ctxt.winit_events.push(event);
                winit::ControlFlow::Break
            });
        }

        // If there are some winit events to process, reset the updates-remaining count.
        if !loop_ctxt.winit_events.is_empty() {
            loop_ctxt.updates_remaining = updates_following_event;
        }

        for winit_event in loop_ctxt.winit_events.drain(..) {
            let (new_model, exit) =
                process_and_emit_winit_event(app, model, loop_ctxt.event_fn, winit_event);
            model = new_model;
            if exit {
                let reason = BreakReason::Exit;
                return Break { model, reason };
            }
        }

        // Update the app's durations.
        let now = Instant::now();
        let since_last = now.duration_since(loop_ctxt.last_update).into();
        let since_start = now.duration_since(loop_ctxt.loop_start).into();
        app.duration.since_start = since_start;
        app.duration.since_prev_update = since_last;
        app.time = app.duration.since_start.secs() as _;

        // Emit an update event.
        let update = update_event(loop_ctxt.loop_start, &mut loop_ctxt.last_update);
        if let Some(event_fn) = loop_ctxt.event_fn {
            let event = E::from(update.clone());
            event_fn(&app, &mut model, event);
        }
        if let Some(update_fn) = loop_ctxt.update_fn {
            update_fn(&app, &mut model, update);
        }
        loop_ctxt.updates_remaining -= 1;

        // Draw to each window.
        for window_id in app.window_ids() {
            acquire_image_and_view_frame(app, window_id, &model, loop_ctxt.default_view.as_ref());
        }

        // Sleep if there's still some time left within the interval.
        let now = Instant::now();
        let since_last_loop_end = now.duration_since(loop_ctxt.last_loop_end);
        if since_last_loop_end < update_interval {
            std::thread::sleep(update_interval - since_last_loop_end);
        }
        loop_ctxt.last_loop_end = Instant::now();

        // See if the loop mode has changed. If so, break.
        match app.loop_mode() {
            LoopMode::Wait { update_interval: ui, updates_following_event: ufe } => {
                update_interval = ui;
                updates_following_event = ufe;
            },
            loop_mode => {
                let reason = BreakReason::NewLoopMode(loop_mode);
                return Break { model, reason };
            }
        };
    }
}

// Run the application loop under the `RefreshSync` mode.
fn run_loop_mode_refresh_sync<M, E>(
    app: &mut App,
    mut model: M,
    loop_ctxt: &mut LoopContext<M, E>,
    mut minimum_update_interval: Duration,
    mut windows: Option<HashSet<window::Id>>,
) -> Break<M>
where
    M: 'static,
    E: LoopEvent,
{
    loop {
        // TODO: Properly consider the impact of an individual window blocking.
        for window_id in app.window_ids() {
            // Skip closed windows.
            if app.window(window_id).is_none() {
                continue;
            }

            cleanup_unused_gpu_resources_for_window(app, window_id);

            // Ensure swapchain dimensions are up to date.
            loop {
                if window_swapchain_needs_recreation(app, window_id) {
                    match recreate_window_swapchain(app, window_id) {
                        Ok(()) => break,
                        Err(SwapchainCreationError::UnsupportedDimensions) => {
                            set_window_swapchain_needs_recreation(app, window_id, true);
                            continue
                        },
                        Err(err) => panic!("{:?}", err),
                    }
                }
                break;
            }

            // Acquire the next image from the swapchain.
            let timeout = None;
            let swapchain = app.windows.borrow()[&window_id].swapchain.clone();
            let next_img = vulkano::swapchain::acquire_next_image(
                swapchain.swapchain.clone(),
                timeout,
            );
            let (swapchain_image_index, swapchain_image_acquire_future) = match next_img {
                Ok(r) => r,
                Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                    set_window_swapchain_needs_recreation(app, window_id, true);
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

            // Process pending app events.
            let (new_model, exit, event_count) = poll_and_process_events(app, model, loop_ctxt);
            model = new_model;
            if exit {
                let reason = BreakReason::Exit;
                return Break { model, reason };
            }

            // Only emit an `update` if there was some user input or if it's been less than
            // `minimum_update_interval`.
            let now = Instant::now();
            let since_last = now.duration_since(loop_ctxt.last_update).into();
            let should_emit_update = windows
                .as_ref()
                .map(|ws| ws.contains(&window_id))
                .unwrap_or(true)
                && (event_count > 0 || since_last > minimum_update_interval);
            if should_emit_update {
                let since_start = now.duration_since(loop_ctxt.loop_start).into();
                app.duration.since_start = since_start;
                app.duration.since_prev_update = since_last;
                app.time = app.duration.since_start.secs() as _;

                // Emit an update event.
                let update = update_event(loop_ctxt.loop_start, &mut loop_ctxt.last_update);
                if let Some(event_fn) = loop_ctxt.event_fn {
                    let event = E::from(update.clone());
                    event_fn(&app, &mut model, event);
                }
                if let Some(update_fn) = loop_ctxt.update_fn {
                    update_fn(&app, &mut model, update);
                }
            }

            // If the window has been removed, continue.
            if app.window(window_id).is_none() {
                continue;
            }

            view_frame(
                app,
                &model,
                window_id,
                swapchain_image_index,
                swapchain_image_acquire_future,
                loop_ctxt.default_view.as_ref(),
            );
        }

        loop_ctxt.last_loop_end = Instant::now();

        // See if the loop mode has changed. If so, break.
        match app.loop_mode() {
            LoopMode::RefreshSync { minimum_update_interval: mli, windows: w } => {
                minimum_update_interval = mli;
                windows = w;
            },
            loop_mode => {
                let reason = BreakReason::NewLoopMode(loop_mode);
                return Break { model, reason };
            }
        }
    }
}

// Recreate window swapchains as necessary due to `loop_mode` switch.
fn change_loop_mode(app: &App, loop_mode: &LoopMode) {
    // Re-build the window swapchains so that they are optimal for the new loop mode.
    let mut windows = app.windows.borrow_mut();
    for window in windows.values_mut() {
        change_loop_mode_for_window(window, loop_mode);
    }
}

// Recreate the window swapchain to match the loop mode.
fn change_loop_mode_for_window(window: &mut Window, loop_mode: &LoopMode) {
    let device = window.swapchain.swapchain.device().clone();
    let surface = window.surface.clone();
    let queue = window.queue.clone();

    // Initialise a swapchain builder from the current swapchain's params.
    let mut swapchain_builder =
        window::SwapchainBuilder::from_swapchain(&window.swapchain.swapchain);

    // Let the new present mode and image count be chosen by nannou or the user if
    // they have a preference.
    swapchain_builder.present_mode = window.user_specified_present_mode;
    swapchain_builder.image_count = window.user_specified_image_count;

    // Create the new swapchain.
    let (new_swapchain, new_swapchain_images) = swapchain_builder
        .build(
            device,
            surface,
            &queue,
            &loop_mode,
            None,
            Some(&window.swapchain.swapchain),
        )
        .expect("failed to recreate swapchain for new `LoopMode`");

    // Replace the window's swapchain with the newly created one.
    window.replace_swapchain(new_swapchain, new_swapchain_images);
}

// Each window has its own associated GPU future associated with displaying the last frame.
// This method cleans up any unused resources associated with this GPU future.
fn cleanup_unused_gpu_resources_for_window(app: &App, window_id: window::Id) {
    let windows = app.windows.borrow();
    let mut guard = windows[&window_id]
        .swapchain
        .previous_frame_end
        .lock()
        .expect("failed to lock `previous_frame_end`");
    if let Some(future) = guard.as_mut() {
        future.cleanup_finished();
    }
}

// Returns `true` if the window's swapchain needs to be recreated.
fn window_swapchain_needs_recreation(app: &App, window_id: window::Id) -> bool {
    let windows = app.windows.borrow();
    let window = &windows[&window_id];
    window.swapchain.needs_recreation.load(atomic::Ordering::Relaxed)
}

// Attempt to recreate a window's swapchain with the window's current dimensions.
fn recreate_window_swapchain(
    app: &App,
    window_id: window::Id,
) -> Result<(), SwapchainCreationError> {
    let mut windows = app.windows.borrow_mut();
    let window = windows.get_mut(&window_id).expect("no window for id");

    // Get the new dimensions for the viewport/framebuffers.
    let dimensions = window
        .surface
        .capabilities(window.swapchain.device().physical_device())
        .expect("failed to get surface capabilities")
        .current_extent
        .expect("current_extent was `None`");

    // Recreate the swapchain with the current dimensions.
    let (new_swapchain, new_images) = window
        .swapchain
        .swapchain
        .recreate_with_dimension(dimensions)?;

    // Update the window's swapchain and images.
    window.replace_swapchain(new_swapchain, new_images);

    Ok(())
}

// Shorthand for setting a window's swapchain.needs_recreation atomic bool.
fn set_window_swapchain_needs_recreation(app: &App, window_id: window::Id, b: bool) {
    let windows = app.windows.borrow_mut();
    let window = windows.get(&window_id).expect("no window for id");
    window.swapchain.needs_recreation.store(b, atomic::Ordering::Relaxed);
}

// Poll and process any pending application events.
//
// Returns:
//
// - the resulting state of the model.
// - whether or not an event should cause exiting the application loop.
// - the number of winit events processed processed.
fn poll_and_process_events<M, E>(
    app: &mut App,
    mut model: M,
    loop_ctxt: &mut LoopContext<M, E>,
) -> (M, bool, usize)
where
    M: 'static,
    E: LoopEvent,
{
    let mut event_count = 0;
    app.events_loop.poll_events(|event| loop_ctxt.winit_events.push(event));
    for winit_event in loop_ctxt.winit_events.drain(..) {
        event_count += 1;
        let (new_model, exit) = process_and_emit_winit_event(
            app,
            model,
            loop_ctxt.event_fn,
            winit_event,
        );
        model = new_model;
        if exit {
            return (model, exit, event_count);
        }
    }
    (model, false, event_count)
}

// Whether or not the given event should toggle fullscreen.
fn should_toggle_fullscreen(winit_event: &winit::WindowEvent) -> bool {
    let input = match *winit_event {
        winit::WindowEvent::KeyboardInput { ref input, .. } => match input.state {
            event::ElementState::Pressed => input,
            _ => return false,
        },
        _ => return false,
    };

    let key = match input.virtual_keycode {
        None => return false,
        Some(k) => k,
    };
    let mods = &input.modifiers;

    // On linux, check for the F11 key (with no modifiers down).
    //
    // TODO: Somehow add special case for KDE?
    if cfg!(target_os = "linux") {
        if !mods.logo && !mods.shift && !mods.alt && !mods.ctrl {
            if let Key::F11 = key {
                return true;
            }
        }

    // On macos and windows check for the logo key plus `f` with no other modifiers.
    } else if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        if mods.logo && !mods.shift && !mods.alt && !mods.ctrl {
            if let Key::F = key {
                return true;
            }
        }
    }

    false
}

// A function to simplify the creation of an `Update` event.
//
// Also updates the given `last_update` instant to `Instant::now()`.
fn update_event(loop_start: Instant, last_update: &mut Instant) -> event::Update {
    // Emit an update event.
    let now = Instant::now();
    let since_last = now.duration_since(*last_update).into();
    let since_start = now.duration_since(loop_start).into();
    let update = event::Update {
        since_last,
        since_start,
    };
    *last_update = now;
    update
}

// Event handling boilerplate shared between the `Rate` and `Wait` loop modes.
//
// 1. Checks for exit on escape.
// 2. Removes closed windows from app.
// 3. Emits event via `event_fn`.
// 4. Returns whether or not we should break from the loop.
fn process_and_emit_winit_event<M, E>(
    app: &mut App,
    mut model: M,
    event_fn: Option<EventFn<M, E>>,
    winit_event: winit::Event,
) -> (M, bool)
where
    M: 'static,
    E: LoopEvent,
{
    // Inspect the event to see if it would require closing the App.
    let mut exit_on_escape = false;
    let mut removed_window = None;
    if let winit::Event::WindowEvent {
        window_id,
        ref event,
    } = winit_event
    {
        // If we should exit the app on escape, check for the escape key.
        if app.exit_on_escape() {
            if let winit::WindowEvent::KeyboardInput { input, .. } = *event {
                if let Some(Key::Escape) = input.virtual_keycode {
                    exit_on_escape = true;
                }
            }
        }

        // If a window was destroyed, remove it from the display map.
        if let winit::WindowEvent::Destroyed = *event {
            removed_window = app.windows.borrow_mut().remove(&window_id);
        // TODO: We should allow the user to handle this case. E.g. allow for doing things like
        // "would you like to save".
        } else if let winit::WindowEvent::CloseRequested = *event {
            removed_window = app.windows.borrow_mut().remove(&window_id);
        } else {
            // Get the size of the screen for translating coords and dimensions.
            let (win_w_px, win_h_px, hidpi_factor) = match app.window(window_id) {
                Some(win) => {
                    // If we should toggle fullscreen for this window, do so.
                    if app.fullscreen_on_shortcut() {
                        if should_toggle_fullscreen(event) {
                            if win.is_fullscreen() {
                                win.set_fullscreen(None);
                            } else {
                                let monitor = win.current_monitor();
                                win.set_fullscreen(Some(monitor));
                            }
                        }
                    }

                    let (w_px, h_px) = win.inner_size_pixels();
                    let hidpi_factor = win.hidpi_factor();
                    (w_px, h_px, hidpi_factor)
                }
                None => (0, 0, 1.0),
            };

            // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
            //
            // winit produces input events in pixels, so these positions need to be divided by the
            // width and height of the window in order to be DPI agnostic.
            let tx = |x: geom::scalar::Default| {
                (x - win_w_px as geom::scalar::Default / 2.0) / hidpi_factor
            };
            let ty = |y: geom::scalar::Default| {
                -((y - win_h_px as geom::scalar::Default / 2.0) / hidpi_factor)
            };

            // If the window ID has changed, ensure the dimensions are up to date.
            if *app.focused_window.borrow() != Some(window_id) {
                if app.window(window_id).is_some() {
                    *app.focused_window.borrow_mut() = Some(window_id);
                }
            }

            // Check for events that would update either mouse, keyboard or window state.
            match *event {
                winit::WindowEvent::CursorMoved {
                    position, ..
                } => {
                    let (x, y): (f64, f64) = position.into();
                    let x = tx(x as _);
                    let y = ty(y as _);
                    app.mouse.x = x;
                    app.mouse.y = y;
                    app.mouse.window = Some(window_id);
                }

                winit::WindowEvent::MouseInput { state, button, .. } => {
                    match state {
                        event::ElementState::Pressed => {
                            let p = app.mouse.position();
                            app.mouse.buttons.press(button, p);
                        }
                        event::ElementState::Released => {
                            app.mouse.buttons.release(button);
                        }
                    }
                    app.mouse.window = Some(window_id);
                }

                winit::WindowEvent::KeyboardInput { input, .. } => {
                    app.keys.mods = input.modifiers;
                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            event::ElementState::Pressed => {
                                app.keys.down.keys.insert(key);
                            }
                            event::ElementState::Released => {
                                app.keys.down.keys.remove(&key);
                            }
                        }
                    }
                }

                _ => (),
            }

            // See if the event could be interpreted as a `ui::Input`. If so, submit it to the
            // `Ui`s associated with this window.
            if let Some(window) = app.windows.borrow().get(&window_id) {
                if let Some(input) = ui::winit_window_event_to_input(event.clone(), window) {
                    if let Some(handles) = app.ui.windows.borrow().get(&window_id) {
                        for handle in handles {
                            if let Some(ref tx) = handle.input_tx {
                                tx.try_send(input.clone()).ok();
                            }
                        }
                    }
                }
            }
        }
    }

    // If the user provided an event function and winit::Event could be interpreted as some event
    // `E`, use it to update the model.
    if let Some(event_fn) = event_fn {
        if let Some(event) = E::from_winit_event(winit_event.clone(), app) {
            event_fn(&app, &mut model, event);
        }
    }

    // If the event was a window event, and the user specified an event function for this window,
    // call it.
    if let winit::Event::WindowEvent { window_id, event } = winit_event {
        // Raw window events.
        if let Some(raw_window_event_fn) = {
            let windows = app.windows.borrow();
            windows
                .get(&window_id)
                .and_then(|w| w.user_functions.raw_event.clone())
                .or_else(|| {
                    removed_window
                        .as_ref()
                        .and_then(|w| w.user_functions.raw_event.clone())
                })
        } {
            let raw_window_event_fn = raw_window_event_fn
                .to_fn_ptr::<M>()
                .expect("unexpected model argument given to window event function");
            (*raw_window_event_fn)(&app, &mut model, event.clone());
        }

        let (win_w, win_h) = {
            let windows = app.windows.borrow();
            windows
                .get(&window_id)
                .and_then(|w| {
                    w.surface
                        .window()
                        .get_inner_size()
                        .map(|size| size.into())
                })
                .unwrap_or((0f64, 0f64))
        };

        // If the event can be represented by a simplified nannou event, check for relevant user
        // functions to be called.
        if let Some(simple) = event::WindowEvent::from_winit_window_event(event, win_w, win_h) {
            // Nannou window events.
            if let Some(window_event_fn) = {
                let windows = app.windows.borrow();
                windows
                    .get(&window_id)
                    .and_then(|w| w.user_functions.event.clone())
                    .or_else(|| {
                        removed_window
                            .as_ref()
                            .and_then(|w| w.user_functions.event.clone())
                    })
            } {
                let window_event_fn = window_event_fn
                    .to_fn_ptr::<M>()
                    .expect("unexpected model argument given to window event function");
                (*window_event_fn)(&app, &mut model, simple.clone());
            }

            // A macro to simplify calling event-specific user functions.
            macro_rules! call_user_function {
                ($fn_name:ident $(,$arg:expr)*) => {{
                    if let Some(event_fn) = {
                        let windows = app.windows.borrow();
                        windows
                            .get(&window_id)
                            .and_then(|w| w.user_functions.$fn_name.clone())
                            .or_else(|| {
                                removed_window
                                    .as_ref()
                                    .and_then(|w| w.user_functions.$fn_name.clone())
                            })
                    } {
                        let event_fn = event_fn
                            .to_fn_ptr::<M>()
                            .unwrap_or_else(|| {
                                panic!(
                                    "unexpected model argument given to {} function",
                                    stringify!($fn_name),
                                );
                            });
                        (*event_fn)(&app, &mut model, $($arg),*);
                    }
                }};
            }

            // Check for more specific event functions.
            match simple {
                event::WindowEvent::KeyPressed(key) => {
                    call_user_function!(key_pressed, key)
                }
                event::WindowEvent::KeyReleased(key) => {
                    call_user_function!(key_released, key)
                }
                event::WindowEvent::MouseMoved(pos) => {
                    call_user_function!(mouse_moved, pos)
                }
                event::WindowEvent::MousePressed(button) => {
                    call_user_function!(mouse_pressed, button)
                }
                event::WindowEvent::MouseReleased(button) => {
                    call_user_function!(mouse_released, button)
                }
                event::WindowEvent::MouseEntered => {
                    call_user_function!(mouse_entered)
                }
                event::WindowEvent::MouseExited => {
                    call_user_function!(mouse_exited)
                }
                event::WindowEvent::MouseWheel(amount, phase) => {
                    call_user_function!(mouse_wheel, amount, phase)
                }
                event::WindowEvent::Moved(pos) => {
                    call_user_function!(moved, pos)
                }
                event::WindowEvent::Resized(size) => {
                    call_user_function!(resized, size)
                }
                event::WindowEvent::Touch(touch) => {
                    call_user_function!(touch, touch)
                }
                event::WindowEvent::TouchPressure(pressure) => {
                    call_user_function!(touchpad_pressure, pressure)
                }
                event::WindowEvent::HoveredFile(path) => {
                    call_user_function!(hovered_file, path)
                }
                event::WindowEvent::HoveredFileCancelled => {
                    call_user_function!(hovered_file_cancelled)
                }
                event::WindowEvent::DroppedFile(path) => {
                    call_user_function!(dropped_file, path)
                }
                event::WindowEvent::Focused => {
                    call_user_function!(focused)
                }
                event::WindowEvent::Unfocused => {
                    call_user_function!(unfocused)
                }
                event::WindowEvent::Closed => {
                    call_user_function!(closed)
                }
            }
        }
    }

    // If exit on escape was triggered, we're done.
    let exit = if exit_on_escape || app.windows.borrow().is_empty() {
        true
    } else {
        false
    };

    (model, exit)
}

// Draw the state of the model to the swapchain image associated with the given index..
//
// This calls the `view` function specified by the user.
fn view_frame<M>(
    app: &mut App,
    model: &M,
    window_id: window::Id,
    swapchain_image_index: usize,
    swapchain_image_acquire_future: window::SwapchainAcquireFuture,
    default_view: Option<&View<M>>,
)
where
    M: 'static,
{
    // Retrieve the queue and swapchain associated with this window.
    let (queue, swapchain) = {
        let windows = app.windows.borrow();
        let window = &windows[&window_id];
        (window.queue.clone(), window.swapchain.clone())
    };

    // Draw the state of the model to the screen.
    let (swapchain_image, nth_frame, swapchain_frame_created) = {
        let mut windows = app.windows.borrow_mut();
        let window = windows.get_mut(&window_id).expect("no window for id");
        let swapchain_image = window.swapchain.images[swapchain_image_index].clone();
        let frame_count = window.frame_count;
        let swapchain_frame_created = window.swapchain.frame_created;
        window.frame_count += 1;
        (swapchain_image, frame_count, swapchain_frame_created)
    };

    // Construct and emit a frame via `view` for receiving the user's graphics commands.
    let frame = Frame::new_empty(
        queue.clone(),
        window_id,
        nth_frame,
        swapchain_image_index,
        swapchain_image,
        swapchain_frame_created,
    ).expect("failed to create `Frame`");
    // If the user specified a view function specifically for this window, use it.
    // Otherwise, use the fallback, default view passed to the app if there was one.
    let window_view = {
        let windows = app.windows.borrow();
        windows
            .get(&window_id)
            .and_then(|w| w.user_functions.view.clone())
    };
    let frame = match window_view {
        Some(window::View::Sketch(view)) => view(app, frame),
        Some(window::View::WithModel(view)) => {
            let view = view
                .to_fn_ptr::<M>()
                .expect("unexpected model argument given to window view function");
            (*view)(app, model, frame)
        },
        None => match default_view {
            Some(View::Sketch(view)) => view(app, frame),
            Some(View::WithModel(view)) => view(app, &model, frame),
            None => frame,
        },
    };
    let command_buffer = frame.finish().build().expect("failed to build command buffer");

    let mut windows = app.windows.borrow_mut();
    let window = windows.get_mut(&window_id).expect("no window for id");

    // Wait for the previous frame presentation to be finished to avoid out-pacing the GPU on macos.
    if let Some(mut previous_frame_fence_signal_future) = window
        .swapchain
        .previous_frame_end
        .lock()
        .expect("failed to lock `previous_frame_end`")
        .take()
    {
        previous_frame_fence_signal_future.cleanup_finished();
        previous_frame_fence_signal_future
            .wait(None)
            .expect("failed to wait for `previous_frame_end` future to signal fence");
    }

    // The future associated with the end of the current frame.
    let future_result = {
        let present_future = swapchain_image_acquire_future
            .then_execute(queue.clone(), command_buffer)
            .expect("failed to execute future")
            .then_swapchain_present(
                queue.clone(),
                swapchain.swapchain.clone(),
                swapchain_image_index,
            );
        (Box::new(present_future) as Box<GpuFuture>)
            .then_signal_fence_and_flush()
    };

    // Handle the result of the future.
    let current_frame_end = match future_result {
        Ok(future) => Some(future),
        Err(vulkano::sync::FlushError::OutOfDate) => {
            window.swapchain.needs_recreation.store(true, atomic::Ordering::Relaxed);
            None
        }
        Err(e) => {
            println!("{:?}", e);
            None
        }
    };

    *window
        .swapchain
        .previous_frame_end
        .lock()
        .expect("failed to acquire `previous_frame_end` lock") = current_frame_end;
}

// Acquire the next swapchain image for the given window and draw to it using the user's
// view function.
//
// Returns whether or not `view_frame` was called. This will be `false` if the given window
// no longer exists or if the swapchain is `OutOfDate` and needs recreation by the time
// `acquire_next_image` is called.
fn acquire_image_and_view_frame<M>(
    app: &mut App,
    window_id: window::Id,
    model: &M,
    view: Option<&View<M>>,
) -> bool
where
    M: 'static,
{
    // Skip closed windows.
    if app.window(window_id).is_none() {
        return false;
    }
    cleanup_unused_gpu_resources_for_window(app, window_id);

    // Ensure swapchain dimensions are up to date.
    loop {
        if window_swapchain_needs_recreation(app, window_id) {
            match recreate_window_swapchain(app, window_id) {
                Ok(()) => break,
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    set_window_swapchain_needs_recreation(app, window_id, true);
                    continue
                },
                Err(err) => panic!("{:?}", err),
            }
        }
        break;
    }

    // Acquire the next image from the swapchain.
    let timeout = None;
    let swapchain = app.windows.borrow()[&window_id].swapchain.clone();
    let next_img = vulkano::swapchain::acquire_next_image(
        swapchain.swapchain.clone(),
        timeout,
    );
    let (swapchain_image_index, swapchain_image_acquire_future) = match next_img {
        Ok(r) => r,
        Err(vulkano::swapchain::AcquireError::OutOfDate) => {
            set_window_swapchain_needs_recreation(app, window_id, true);
            return false;
        },
        Err(err) => panic!("{:?}", err)
    };

    view_frame(
        app,
        model,
        window_id,
        swapchain_image_index,
        swapchain_image_acquire_future,
        view,
    );
    true
}
