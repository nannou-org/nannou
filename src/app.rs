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
use event::{self, Event, LoopEvent, Key};
use find_folder;
use frame::Frame;
use geom;
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
use ui;
use vulkano;
use vulkano::device::DeviceOwned;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::GpuFuture;
use window::{self, Window};
use winit;

/// The user function type for initialising their model.
pub type ModelFn<Model> = fn(&App) -> Model;

/// The user function type for updating their model in accordance with some event.
pub type EventFn<Model, Event> = fn(&App, Model, Event) -> Model;

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
    event: EventFn<M, E>,
    view: Option<View<M>>,
    exit: Option<ExitFn<M>>,
    create_default_window: bool,
}

/// The default `model` function used when none is specified by the user.
fn default_model(_: &App) -> () {
    ()
}

/// The default `event` function used when none is specified by the user.
fn default_event<M>(_: &App, model: M, _: Event) -> M {
    model
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
/// - **All OpenGL windows** for graphics and user input. Windows can be referenced via their IDs.
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
    /// The number of times the **App**'s **view** function has been called since the start of the
    /// program.
    ///
    /// TODO: Move this to the window struct (see issue #213).
    pub(crate) elapsed_frames: u64,

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
    event_fn: EventFn<M, E>,
    // The user's default function for drawing to a window's swapchain's image.
    view: Option<View<M>>,
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
    /// if it has been longer than `minimum_latency_interval` or if some user input was received
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
        /// The minimum amount of latency that is allowed to occur between the moment at which
        /// `event` was last called with an `Update` and the moment at which `view` is called by
        /// a window's swapchain.
        minimum_latency_interval: Duration,
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

impl<M> Builder<M, Event> {
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
            event: default_event,
            view: None,
            exit: None,
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
            view,
            exit,
            create_default_window,
            ..
        } = self;
        Builder {
            model,
            event,
            view,
            exit,
            create_default_window,
        }
    }
}

impl<M, E> Builder<M, E>
where
    E: LoopEvent,
{
    /// The `view` function that the app will call to allow you to present your Model to the
    /// surface of a window on your display.
    ///
    /// Note that when working with more than one window, you can use `frame.window_id()` to
    /// determine which window the current call is associated with.
    pub fn view(mut self, view: ViewFn<M>) -> Self {
        self.view = Some(View::WithModel(view));
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
        self.view = Some(View::WithModel(view));
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

    /// Build and run an `App` with the specified parameters.
    ///
    /// This function will not return until the application has exited.
    ///
    /// If you wish to remain cross-platform frienly, we recommend that you call this on the main
    /// thread as some platforms require that their application event loop and windows are
    /// initialised on the main thread.
    pub fn run(self) {
        // Start the winit window event loop.
        let events_loop = winit::EventsLoop::new();

        // Initialise the app.
        let app = App::new(events_loop).expect("failed to construct `App`");

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

        run_loop(app, model, self.event, self.view, self.exit)
    }
}

impl Builder<(), Event> {
    /// Shorthand for building a simple app that has no model, handles no events and simply draws
    /// to a single window.
    ///
    /// This is useful for late night hack sessions where you just don't care about all that other
    /// stuff, you just want to play around with some ideas or make something pretty.
    pub fn sketch(view: SketchViewFn) {
        let builder = Builder {
            model: default_model,
            event: default_event,
            view: Some(View::Sketch(view)),
            exit: None,
            create_default_window: true,
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
}

impl Default for LoopMode {
    fn default() -> Self {
        //LoopMode::rate_fps(Self::DEFAULT_RATE_FPS)
        LoopMode::RefreshSync {
            minimum_latency_interval: update_interval(Self::DEFAULT_RATE_FPS),
            windows: None,
        }
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
    ) -> Result<Self, vulkano::instance::InstanceCreationError> {
        let vulkan_instance = {
            let app_infos = None;
            let extensions = vulkano_win::required_extensions();
            let layers = None;
            vulkano::instance::Instance::new(app_infos, &extensions, layers)?
        };
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
        let elapsed_frames = 0;
        let app = App {
            vulkan_instance,
            events_loop,
            events_loop_is_asleep,
            focused_window,
            elapsed_frames,
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
        let format = window.swapchain.swapchain.format();
        let draw = self.draw_state.draw.borrow_mut();
        draw.reset();
        if self.draw_state.renderer.borrow().is_none() {
            let renderer = draw::backend::vulkano::Renderer::new(device, format)
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

    /// The number of times the **App**'s **view** function has been called since the start of the
    /// program.
    ///
    /// TODO: Move this to the window struct as **view** is now called per-window (see issue #213).
    pub fn elapsed_frames(&self) -> u64 {
        self.elapsed_frames
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
        renderer.draw_to_frame(&self.draw, dpi_factor, frame)
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
    model: M,
    event_fn: EventFn<M, E>,
    view: Option<View<M>>,
    exit_fn: Option<ExitFn<M>>,
)
where
    E: LoopEvent,
{
    let loop_start = Instant::now();

    // Initialise the loop context.
    let mut loop_ctxt = LoopContext {
        event_fn,
        view,
        loop_start,
        winit_events: vec![],
        last_update: loop_start,
        last_loop_end: loop_start,
        updates_remaining: LoopMode::DEFAULT_UPDATES_FOLLOWING_EVENT,
    };

    let mut loop_mode = app.loop_mode();

    // Begin running the application loop based on the current `LoopMode`.
    'mode: loop {
        let Break { model, reason } = match loop_mode {
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
            LoopMode::RefreshSync { minimum_latency_interval, windows } => {
                run_loop_mode_refresh_sync(
                    &mut app,
                    model,
                    &mut loop_ctxt,
                    minimum_latency_interval,
                    windows,
                )
            }
        };

        match reason {
            // If the break reason was due to the `LoopMode` changing, switch to the new loop mode
            // and continue.
            BreakReason::NewLoopMode(new_loop_mode) => {
                loop_mode = new_loop_mode;
                unimplemented!();
            }
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
    _update_interval: Duration,
) -> Break<M>
where
    E: LoopEvent,
{
    loop {
        // See if the loop mode has changed. If so, break.
        let update_interval = match app.loop_mode() {
            LoopMode::Rate { update_interval } => update_interval,
            loop_mode => {
                let reason = BreakReason::NewLoopMode(loop_mode);
                return Break { model, reason };
            }
        };

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
        let event = E::from(update_event(loop_ctxt.loop_start, &mut loop_ctxt.last_update));
        model = (loop_ctxt.event_fn)(&app, model, event);

        // Draw the state of the model to the screen.
        draw(
            &app,
            &model,
            loop_ctxt.view.as_ref().expect("no default window view"),
        ).unwrap();
        app.elapsed_frames += 1;

        // Sleep if there's still some time left within the interval.
        let now = Instant::now();
        let since_last_loop_end = now.duration_since(loop_ctxt.last_loop_end);
        if since_last_loop_end < update_interval {
            std::thread::sleep(update_interval - since_last_loop_end);
        }
        loop_ctxt.last_loop_end = Instant::now();
    }
}

// Run the application loop under the `Wait` mode.
fn run_loop_mode_wait<M, E>(
    app: &mut App,
    mut model: M,
    loop_ctxt: &mut LoopContext<M, E>,
    _updates_following_event: usize,
    _update_interval: Duration,
) -> Break<M>
where
    E: LoopEvent,
{
    loop {
        // See if the loop mode has changed. If so, break.
        let (update_interval, updates_following_event) = match app.loop_mode() {
            LoopMode::Wait { update_interval, updates_following_event } => {
                (update_interval, updates_following_event)
            },
            loop_mode => {
                let reason = BreakReason::NewLoopMode(loop_mode);
                return Break { model, reason };
            }
        };

        // First collect any pending window events.
        app.events_loop
            .poll_events(|event| loop_ctxt.winit_events.push(event));

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
        let event = E::from(update_event(loop_ctxt.loop_start, &mut loop_ctxt.last_update));
        model = (loop_ctxt.event_fn)(&app, model, event);
        loop_ctxt.updates_remaining -= 1;

        // Draw the state of the model to the screen.
        draw(
            &app,
            &model,
            loop_ctxt.view.as_ref().expect("no default window view"),
        ).unwrap();
        app.elapsed_frames += 1;

        // Sleep if there's still some time left within the interval.
        let now = Instant::now();
        let since_last_loop_end = now.duration_since(loop_ctxt.last_loop_end);
        if since_last_loop_end < update_interval {
            std::thread::sleep(update_interval - since_last_loop_end);
        }
        loop_ctxt.last_loop_end = Instant::now();
    }
}

// Run the application loop under the `RefreshSync` mode.
fn run_loop_mode_refresh_sync<M, E>(
    app: &mut App,
    mut model: M,
    loop_ctxt: &mut LoopContext<M, E>,
    mut minimum_latency_interval: Duration,
    mut windows: Option<HashSet<window::Id>>,
) -> Break<M>
where
    E: LoopEvent,
{
    loop {
        // See if the loop mode has changed. If so, break.
        let (minimum_latency_interval, windows) = match app.loop_mode() {
            LoopMode::RefreshSync { minimum_latency_interval, windows } => {
                (minimum_latency_interval, windows)
            },
            loop_mode => {
                let reason = BreakReason::NewLoopMode(loop_mode);
                return Break { model, reason };
            }
        };

        // TODO: Properly consider the impact of an individual window blocking.
        let windows = {
            let windows = app.windows.borrow();
            windows.iter()
                .map(|(&id, window)| (id, window.queue.clone()))
                .collect::<Vec<_>>()
        };
        for (window_id, queue) in windows {
            // Skip closed windows and cleanup unused GPU resources.
            {
                let windows = app.windows.borrow();
                let window = match windows.get(&window_id) {
                    Some(w) => w,
                    None => continue,
                };
                windows[&window_id]
                    .swapchain
                    .previous_frame_end
                    .lock()
                    .expect("failed to lock `previous_frame_end`")
                    .as_mut()
                    .expect("`previous_frame_end` was `None`")
                    .cleanup_finished();
            }

            // Swapchain Recreation
            //
            // If the swapchain requires recreation, we must do the following:
            //
            // - Retrieve the `current_extent` of the window surface capabilities.
            // - Recreate the swapchain and its images with the current_extent.
            // - Recreate framebuffers.
            // - Update the `viewports` of the dynamic state.
            // - Signal recreation is complete.
            let recreate_swapchain = {
                let windows = app.windows.borrow();
                let window = &windows[&window_id];
                window.swapchain.needs_recreation.load(atomic::Ordering::Relaxed)
            };
            if recreate_swapchain {
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
                let new_swapchain = window.swapchain.swapchain.recreate_with_dimension(dimensions);
                let (new_swapchain, new_images) = match new_swapchain {
                    Ok(r) => r,
                    // This error tends to happen when the user is manually resizing the window.
                    // Simply restarting the loop is the easiest way to fix this issue.
                    Err(SwapchainCreationError::UnsupportedDimensions) => {
                        continue;
                    },
                    Err(err) => panic!("{:?}", err)
                };

                // Update the window's swapchain and images.
                window.replace_swapchain(new_swapchain, new_images);

                // TODO
                //framebuffers = None;

                //dynamic_state.viewports = Some(vec![Viewport {
                //    origin: [0.0, 0.0],
                //    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                //    depth_range: 0.0 .. 1.0,
                //}]);
            }

            // Acquire the next image from the swapchain.
            let timeout = None;
            let swapchain = app.windows.borrow()[&window_id].swapchain.clone();
            let next_img = vulkano::swapchain::acquire_next_image(swapchain.swapchain.clone(), timeout);
            let (image_num, acquire_future) = match next_img {
                Ok(r) => r,
                Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                    let mut windows = app.windows.borrow_mut();
                    let window = windows.get_mut(&window_id).expect("no window for id");
                    window.swapchain.needs_recreation.store(true, atomic::Ordering::Relaxed);
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

            // Process pending app events.
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
            let event = E::from(update_event(loop_ctxt.loop_start, &mut loop_ctxt.last_update));
            model = (loop_ctxt.event_fn)(&app, model, event);

            // If the window has been removed, continue.
            if !app.windows.borrow().contains_key(&window_id) {
                continue;
            }

            // Draw the state of the model to the screen.
            let (swapchain_image, nth_frame) = {
                let mut windows = app.windows.borrow_mut();
                let window = windows.get_mut(&window_id).expect("no window for id");
                let swapchain_image = window.swapchain.images[image_num].clone();
                let frame_count = window.frame_count;
                window.frame_count += 1;
                (swapchain_image, frame_count)
            };
            let frame = Frame::new_empty(
                queue.clone(),
                window_id,
                nth_frame,
                image_num,
                swapchain_image,
            ).expect("failed to create `Frame`");
            let frame = match loop_ctxt.view.as_ref().expect("no default window view") {
                View::Sketch(view) => view(app, frame),
                View::WithModel(view) => view(app, &model, frame),
            };
            app.elapsed_frames += 1;
            let command_buffer = frame.finish().build().expect("failed to build command buffer");

            let mut windows = app.windows.borrow_mut();
            let window = windows.get_mut(&window_id).expect("no window for id");
            let future = window
                .swapchain
                .previous_frame_end
                .lock()
                .expect("failed to lock `previous_frame_end`")
                .take()
                .expect("`previous_frame_end` was `None`")
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffer)
                .expect("failed to execute future")
                // The image color output is now expected to contain the user's graphics.
                // But in order to show it on the screen, we have to `present` the image.
                .then_swapchain_present(queue.clone(), swapchain.swapchain.clone(), image_num)
                // Flush forwards the future to the GPU to begin the actual processing.
                .then_signal_fence_and_flush();
            let previous_frame_end = match future {
                Ok(future) => {
                    Some(Box::new(future) as Box<_>)
                }
                Err(vulkano::sync::FlushError::OutOfDate) => {
                    window.swapchain.needs_recreation.store(true, atomic::Ordering::Relaxed);
                    Some(Box::new(vulkano::sync::now(queue.device().clone())) as Box<_>)
                }
                Err(e) => {
                    println!("{:?}", e);
                    Some(Box::new(vulkano::sync::now(queue.device().clone())) as Box<_>)
                }
            };
            *window
                .swapchain
                .previous_frame_end
                .lock()
                .expect("failed to acquire `previous_frame_end` lock") = previous_frame_end;
        }

        loop_ctxt.last_loop_end = Instant::now();
    }
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

// A function to re-use when drawing for each of the loop modes.
fn draw<M>(app: &App, model: &M, view: &View<M>) -> Result<(), ()> {
    // Draw the state of the model to the screen.
    // let gl_frames = app
    //     .windows
    //     .borrow()
    //     .iter()
    //     .map(|(&id, window)| {
    //         let gl_frame = RefCell::new(frame::GlFrame::new(window.display.draw()));
    //         (id, gl_frame)
    //     })
    //     .collect();
    // TODO: This currently passes the *focused* window but should pass the *main* one.
    //let undrawn_frame = frame::new(gl_frames, *app.focused_window.borrow());
    let undrawn_frame: Frame = unimplemented!();
    let frame = match *view {
        View::WithModel(view_fn) => view_fn(&app, &model, undrawn_frame),
        View::Sketch(view_fn) => view_fn(&app, undrawn_frame),
    };
    //frame::finish(frame)
    Ok(())
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
    event_fn: EventFn<M, E>,
    winit_event: winit::Event,
) -> (M, bool)
where
    E: LoopEvent,
{
    // Inspect the event to see if it would require closing the App.
    let mut exit_on_escape = false;
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
            app.windows.borrow_mut().remove(&window_id);
        // TODO: We should allow the user to handle this case. E.g. allow for doing things like
        // "would you like to save".
        } else if let winit::WindowEvent::CloseRequested = *event {
            app.windows.borrow_mut().remove(&window_id);
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

    // If the winit::Event could be interpreted as some event `E`, use it to update the model.
    if let Some(event) = E::from_winit_event(winit_event, app) {
        model = event_fn(&app, model, event);
    }

    // If exit on escape was triggered, we're done.
    let exit = if exit_on_escape || app.windows.borrow().is_empty() {
        true
    } else {
        false
    };

    (model, exit)
}





    // // A threaded function for polling the window swapchains for available images.
    // //
    // // TODO:
    // //
    // // Sync the swapchains on this thread when windows are created/destroyed via `App`. Possibly
    // // share the swapchains hashmap between threads with an Arc<Mutex<>>? Consider the following:
    // //   - The swapchain (and in turn images) might need to be recreated at any time (e.g. resize).
    // //   - When recreated, swapchain needs current `window` dimensions.
    // //   - A user may create or destroy a window via the `App` at any time.
    // fn run_vulkan_swapchains(
    //     swapchains: HashMap<window::Id, Arc<WindowSwapchain>>,
    // ) -> Result<(), vulkano::swapchain::AcquireError> {
    //     // NOTE: Temporary just to get things working with one window for now.
    //     let (window_id, swapchain) = swapchains.into_iter().next().expect("no window swapchains");

    //     // The device with which the swapchain is associated with.
    //     let device = swapchain.device();

    //     // TODO:
    //     // Whether or not we need to recreate the swapchain due to resizing, etc.
    //     let mut recreate_swapchain = false;

    //     // Hold onto the previous frame as `drop` blocks until the GPU has finished executing it.
    //     let mut previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;

    //     loop {
    //         // Clean up resources associated with commands that have finished processing.
    //         previous_frame_end.cleanup_finished();

    //         // TODO: Recreate swapchain.
    //         if recreate_swapchain {
    //             unimplemented!();
    //         }

    //         // Acquire an image from the swapchain.
    //         let next = swapchain::acquire_next_image(swapchain.clone(), None);
    //         let (image_num, acquire_future) = match next {
    //             Ok(next) => next,
    //             Err(AcquireError::OutOfDate) => {
    //                 recreate_swapchain = true;
    //                 continue;
    //             }
    //             Err(err) => {
    //                 eprintln!("swapchain::acquire_next_image failed with: {}", err);
    //                 return Err(err);
    //             }
    //         };


    // }

    // // Spawn a thread for acquiring window swapchain images for rendering via Vulkan.
    // //
    // // The reason we spawn a thread here is that the `swapchain::acquire_next_image` function may
    // // or may not block depending on the platform implementation. To avoid locking up the main
    // // thread due to this, we acquire swapchain images on a separate thread.
    // let vulkan_swapchain_thread = thread::Builder::new()
    //     .name("vulkan-swapchains".to_string())
    //     .spawn(move || {
    //         run_vulkan_swapchains(windows)
    //     })
    //     .expect("failed to spawn `vulkan-swapchains` thread");


