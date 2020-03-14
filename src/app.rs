//! Items related to the `App` type and the application context in general.
//!
//! See here for items relating to the event loop, device access, creating and managing windows,
//! streams and more.
//!
//! - [**App**](./struct.App.html) - provides a context and API for windowing, devices, etc.
//! - [**Proxy**](./struct.Proxy.html) - a handle to an **App** that may be used from a non-main
//!   thread.
//! - [**Draw**](./struct.Draw.html) - a simple API for drawing graphics, accessible via
//!   `app.draw()`.
//! - [**LoopMode**](./enum.LoopMode.html) - describes the behaviour of the application event loop.

use crate::draw;
use crate::event::{self, Event, Key, LoopEvent, Update};
use crate::frame::{Frame, RawFrame};
use crate::geom;
use crate::state;
use crate::time::DurationF64;
use crate::ui;
use crate::wgpu;
use crate::window::{self, Window};
use find_folder;
use std;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit;
use winit::event_loop::ControlFlow;

/// The user function type for initialising their model.
pub type ModelFn<Model> = fn(&App) -> Model;

/// The user function type for updating their model in accordance with some event.
pub type EventFn<Model, Event> = fn(&App, &mut Model, Event);

/// The user function type for updating the user model within the application loop.
pub type UpdateFn<Model> = fn(&App, &mut Model, Update);

/// The user function type for drawing their model to the surface of a single window.
pub type ViewFn<Model> = fn(&App, &Model, Frame);

/// A shorthand version of `ViewFn` for sketches where the user does not need a model.
pub type SketchViewFn = fn(&App, Frame);

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
    create_default_window: bool,
    default_window_size: Option<winit::dpi::LogicalSize<u32>>,
}

/// A nannou `Sketch` builder.
pub struct SketchBuilder<E = Event> {
    builder: Builder<(), E>,
}

/// The default `model` function used when none is specified by the user.
fn default_model(_: &App) -> () {
    ()
}

/// Each nannou application has a single **App** instance. This **App** represents the entire
/// context of the application.
///
/// The **App** provides access to most application, windowing and "IO" related APIs. In other
/// words, if you need access to windowing, the active wgpu devices, etc, the **App** will provide
/// access to this.
///
/// The **App** owns and manages:
///
/// - The **window and input event loop** used to drive the application forward.
/// - **All windows** for graphics and user input. Windows can be referenced via their IDs.
/// - The sharing of wgpu devices between windows.
/// - A default **Draw** instance for ease of use.
/// - A map of channels for submitting user input updates to active **Ui**s.
pub struct App {
    config: RefCell<Config>,
    default_window_size: Option<winit::dpi::LogicalSize<u32>>,
    pub(crate) event_loop_window_target: Option<EventLoopWindowTarget>,
    pub(crate) event_loop_proxy: Proxy,
    pub(crate) windows: RefCell<HashMap<window::Id, Window>>,
    /// A map of active wgpu physial device adapters.
    adapters: wgpu::AdapterMap,
    draw_state: DrawState,
    pub(crate) ui: ui::Arrangement,
    /// The window that is currently in focus.
    pub(crate) focused_window: RefCell<Option<window::Id>>,
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
    renderer: RefMut<'a, RefCell<draw::backend::wgpu::Renderer>>,
}

// Draw state managed by the **App**.
#[derive(Debug)]
struct DrawState {
    draw: RefCell<draw::Draw<DrawScalar>>,
    renderers: RefCell<HashMap<window::Id, RefCell<draw::backend::wgpu::Renderer>>>,
}

/// The app uses a set scalar type in order to provide a simplistic API to users.
///
/// If you require changing the scalar type to something else, consider using a custom
/// **nannou::draw::Draw** instance.
pub type DrawScalar = geom::scalar::Default;

/// A handle to the **App** that can be shared across threads. This may be used to "wake up" the
/// **App**'s inner event loop.
#[derive(Clone)]
pub struct Proxy {
    event_loop_proxy: winit::event_loop::EventLoopProxy<()>,
    // Indicates whether or not the events loop is currently asleep.
    //
    // This is set to `true` each time the events loop is ready to return and the `LoopMode` is
    // set to `Wait` for events.
    //
    // This value is set back to `false` each time the events loop receives any kind of event.
    event_loop_is_asleep: Arc<AtomicBool>,
}

// State related specifically to the application loop, shared between loop modes.
struct LoopState {
    updates_since_event: usize,
    loop_start: Instant,
    last_update: Instant,
    total_updates: u64,
}

/// The mode in which the **App** is currently running the event loop and emitting `Update` events.
#[derive(Clone, Debug, PartialEq)]
pub enum LoopMode {
    /// Synchronises `Update` events with requests for a new image by the swap chain.
    ///
    /// The result of using this loop mode is similar to using vsync in traditional applications.
    /// E.g. if you have one window running on a monitor with a 60hz refresh rate, your update will
    /// get called at a fairly consistent interval that is close to 60 times per second.
    RefreshSync,

    /// Specifies that the application is continuously looping at a consistent rate.
    ///
    /// **NOTE:** This currently behaves the same as `RefreshSync`. Need to upate this to handled a
    /// fix step properly in the future. See #456.
    Rate {
        /// The minimum interval between emitted updates.
        update_interval: Duration,
    },

    /// Waits for user input events to occur before calling `event` with an `Update` event.
    ///
    /// This is particularly useful for low-energy GUIs that only need to update when some sort of
    /// input has occurred. The benefit of using this mode is that you don't waste CPU cycles
    /// looping or updating when you know nothing is changing in your model or view.
    Wait,

    /// Loops for the given number of updates and then finishes.
    ///
    /// This is similar to the **Wait** loop mode, except that windowing, application and input
    /// events will not cause the loop to update or view again after the initial
    /// `number_of_updates` have already been applied.
    ///
    /// This is useful for sketches where you only want to draw one frame, or if you know exactly
    /// how many updates you require for an animation, etc.
    NTimes {
        /// The number of updates that must be emited regardless of non-update events
        number_of_updates: usize,
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
            create_default_window: false,
            default_window_size: None,
        }
    }

    /// The function that the app will call to allow you to update your Model on events.
    ///
    /// The `event` function allows you to expect any event type that implements `LoopEvent`,
    /// however nannou also provides a default `Event` type that should cover most use cases. This
    /// event type is an `enum` that describes all the different kinds of I/O events that might
    /// occur during the life of the program. These include things like `Update`s and
    /// `WindowEvent`s such as `KeyPressed`, `MouseMoved`, and so on.
    #[cfg_attr(rustfmt, rustfmt_skip)]
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
            default_window_size,
            ..
        } = self;
        Builder {
            model,
            event: Some(event),
            update,
            default_view,
            exit,
            create_default_window,
            default_window_size,
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

    /// Specify the default window size in points.
    ///
    /// If a window is created and its size is not specified, this size will be used.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.default_window_size = Some(winit::dpi::LogicalSize { width, height });
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
        let event_loop = winit::event_loop::EventLoop::new();

        // Create the proxy used to awaken the event loop.
        let event_loop_proxy = event_loop.create_proxy();
        let event_loop_is_asleep = Arc::new(AtomicBool::new(false));
        let event_loop_proxy = Proxy {
            event_loop_proxy,
            event_loop_is_asleep,
        };

        // Initialise the app.
        let event_loop_window_target = Some(EventLoopWindowTarget::Owned(event_loop));
        let app = App::new(
            event_loop_proxy,
            event_loop_window_target,
            self.default_window_size,
        );

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

        run_loop(
            app,
            model,
            self.event,
            self.update,
            self.default_view,
            self.exit,
        );
    }
}

impl<E> SketchBuilder<E>
where
    E: LoopEvent,
{
    /// The size of the sketch window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.builder = self.builder.size(width, height);
        self
    }

    /// Build and run a `Sketch` with the specified parameters.
    ///
    /// This calls `App::run` internally. See that method for details!
    pub fn run(self) {
        self.builder.run()
    }
}

impl Builder<(), Event> {
    /// Shorthand for building a simple app that has no model, handles no events and simply draws
    /// to a single window.
    ///
    /// This is useful for late night hack sessions where you just don't care about all that other
    /// stuff, you just want to play around with some ideas or make something pretty.
    pub fn sketch(view: SketchViewFn) -> SketchBuilder<Event> {
        let builder: Self = Builder {
            model: default_model,
            event: None,
            update: None,
            default_view: Some(View::Sketch(view)),
            exit: None,
            create_default_window: true,
            default_window_size: None,
        };
        SketchBuilder { builder }
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
    /// The minimum number of updates that will be emitted after an event is triggered in Wait
    /// mode.
    pub const UPDATES_PER_WAIT_EVENT: u32 = 3;

    /// A simplified constructor for the default `RefreshSync` loop mode.
    ///
    /// Assumes a display refresh rate of ~60hz and in turn specifies a `minimum_update_latency` of
    /// ~8.33ms. The `windows` field is set to `None`.
    pub fn refresh_sync() -> Self {
        LoopMode::RefreshSync
    }

    /// Specify the **Rate** mode with the given frames-per-second.
    pub fn rate_fps(fps: f64) -> Self {
        let update_interval = update_interval(fps);
        LoopMode::Rate { update_interval }
    }

    /// Specify the **Wait** mode.
    pub fn wait() -> Self {
        LoopMode::Wait
    }

    /// Specify the **Ntimes** mode with one update
    ///
    /// Waits long enough to ensure loop iteration never occurs faster than the given `max_fps`.
    pub fn loop_ntimes(number_of_updates: usize) -> Self {
        LoopMode::NTimes { number_of_updates }
    }

    /// Specify the **Ntimes** mode with one update
    pub fn loop_once() -> Self {
        Self::loop_ntimes(1)
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
        event_loop_proxy: Proxy,
        event_loop_window_target: Option<EventLoopWindowTarget>,
        default_window_size: Option<winit::dpi::LogicalSize<u32>>,
    ) -> Self {
        let adapters = Default::default();
        let windows = RefCell::new(HashMap::new());
        let draw = RefCell::new(draw::Draw::default());
        let config = RefCell::new(Default::default());
        let renderers = RefCell::new(Default::default());
        let draw_state = DrawState { draw, renderers };
        let focused_window = RefCell::new(None);
        let ui = ui::Arrangement::new();
        let mouse = state::Mouse::new();
        let keys = state::Keys::default();
        let duration = state::Time::default();
        let time = duration.since_start.secs() as _;
        let app = App {
            event_loop_proxy,
            event_loop_window_target,
            default_window_size,
            focused_window,
            adapters,
            windows,
            config,
            draw_state,
            ui,
            mouse,
            keys,
            duration,
            time,
        };
        app
    }

    /// Returns the list of all the monitors available on the system.
    pub fn available_monitors(&self) -> Vec<winit::monitor::MonitorHandle> {
        match self.event_loop_window_target {
            Some(EventLoopWindowTarget::Owned(ref event_loop)) => {
                event_loop.available_monitors().collect()
            }
            _ => {
                let windows = self.windows.borrow();
                match windows.values().next() {
                    None => vec![],
                    Some(window) => window.window.available_monitors().collect(),
                }
            }
        }
    }

    /// Returns the primary monitor of the system.
    pub fn primary_monitor(&self) -> winit::monitor::MonitorHandle {
        match self.event_loop_window_target {
            Some(EventLoopWindowTarget::Owned(ref event_loop)) => event_loop.primary_monitor(),
            _ => {
                let windows = self.windows.borrow();
                match windows.values().next() {
                    None => unimplemented!(
                        "yet to implement a way to get `primary_monitor` if neither \
                         event loop or window can be safely accessed"
                    ),
                    Some(window) => window.window.primary_monitor(),
                }
            }
        }
    }

    /// Find and return the absolute path to the project's `assets` directory.
    ///
    /// This method looks for the assets directory in the following order:
    ///
    /// 1. Checks the same directory as the executable.
    /// 2. Recursively checks exe's parent directories (to a max depth of 5).
    /// 3. Recursively checks exe's children directories (to a max depth of 3).
    pub fn assets_path(&self) -> Result<PathBuf, find_folder::Error> {
        find_assets_path()
    }

    /// The path to the current project directory.
    ///
    /// The current project directory is considered to be the directory containing the cargo
    /// manifest (aka the `Cargo.toml` file).
    ///
    /// **Note:** Be careful not to rely on this directory for apps or sketches that you wish to
    /// distribute! This directory is mostly useful for local sketches, experiments and testing.
    pub fn project_path(&self) -> Result<PathBuf, find_folder::Error> {
        find_project_path()
    }

    /// Begin building a new window.
    pub fn new_window(&self) -> window::Builder {
        let builder = window::Builder::new(self);
        match self.default_window_size {
            Some(size) => builder.size(size.width, size.height),
            None => builder,
        }
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
        self.main_window().rect()
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

    /// Access to the **App**'s inner map of wgpu adapters representing access to physical GPU
    /// devices.
    ///
    /// By maintaining a map of active adapters and their established devices, nannou allows for
    /// devices to be shared based on the desired `RequestAdapterOptions` and `DeviceDescriptor`s.
    ///
    /// For example, when creating new windows with the same set of `RequestAdapterOptions` and
    /// `DeviceDescriptor`s, nannou will automatically share devices between windows where
    /// possible. This allows for sharing GPU resources like **Texture**s and **Buffer**s between
    /// windows.
    pub fn wgpu_adapters(&self) -> &wgpu::AdapterMap {
        &self.adapters
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
    /// The default loop mode is `LoopMode::RefreshSync`.
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
        self.event_loop_proxy.clone()
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

        let draw = self.draw_state.draw.borrow_mut();
        draw.reset();

        let renderers = self.draw_state.renderers.borrow_mut();
        let renderer = RefMut::map(renderers, |renderers| {
            renderers.entry(window_id).or_insert_with(|| {
                let device = window.swap_chain_device();
                let frame_dims: [u32; 2] = window.tracked_state.physical_size.into();
                let msaa_samples = window.msaa_samples();
                let target_format = crate::frame::Frame::TEXTURE_FORMAT;
                let renderer = draw::backend::wgpu::Renderer::new(
                    device,
                    frame_dims,
                    msaa_samples,
                    target_format,
                );
                RefCell::new(renderer)
            })
        });
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

    /// The number of frames that can currently be displayed a second
    pub fn fps(&self) -> f32 {
        self.duration.updates_per_second()
    }

    /// The name of the nannou executable that is currently running.
    pub fn exe_name(&self) -> std::io::Result<String> {
        let string = std::env::current_exe()?
            .file_stem()
            .expect("exe path contained no file stem")
            .to_string_lossy()
            .to_string();
        Ok(string)
    }
}

impl Proxy {
    /// Wake up the application!
    ///
    /// This wakes up the **App**'s inner event loop and causes a user event to be emitted by the
    /// event loop.
    ///
    /// The `app::Proxy` stores a flag in order to track whether or not the `EventLoop` is
    /// currently blocking and waiting for events. This method will only call the underlying
    /// `winit::event_loop::EventLoopProxy::send_event` method if this flag is set to true and will
    /// immediately set the flag to false afterwards. This makes it safe to call the `wakeup`
    /// method as frequently as necessary across methods without causing any underlying OS methods
    /// to be called more than necessary.
    pub fn wakeup(&self) -> Result<(), winit::event_loop::EventLoopClosed<()>> {
        if self.event_loop_is_asleep.load(atomic::Ordering::Relaxed) {
            self.event_loop_proxy.send_event(())?;
            self.event_loop_is_asleep
                .store(false, atomic::Ordering::Relaxed);
        }
        Ok(())
    }
}

impl<'a> Draw<'a> {
    /// Draw the current state of the inner mesh to the given frame.
    pub fn to_frame(&self, app: &App, frame: &Frame) -> Result<(), draw::backend::wgpu::DrawError> {
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
        let scale_factor = window.tracked_state.scale_factor as _;
        let mut renderer = self.renderer.borrow_mut();
        renderer.render_to_frame(window.swap_chain_device(), &self.draw, scale_factor, frame);
        Ok(())
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

/// Attempt to find the assets directory path relative to the executable location.
pub fn find_assets_path() -> Result<PathBuf, find_folder::Error> {
    let exe_path = std::env::current_exe()?;
    find_folder::Search::ParentsThenKids(5, 3)
        .of(exe_path
            .parent()
            .expect("executable has no parent directory to search")
            .into())
        .for_folder(App::ASSETS_DIRECTORY_NAME)
}

/// Attempt to find the assets directory path relative to the executable location.
pub fn find_project_path() -> Result<PathBuf, find_folder::Error> {
    let exe_path = std::env::current_exe()?;
    let mut path = exe_path.parent().expect("exe has no parent directory");
    while let Some(parent) = path.parent() {
        path = parent;
        if path.join("Cargo").with_extension("toml").exists() {
            return Ok(path.to_path_buf());
        }
    }
    Err(find_folder::Error::NotFound)
}

// This type allows the `App` to provide an API for creating new windows.
//
// During the `setup` before the
pub(crate) enum EventLoopWindowTarget {
    // Ownership over the event loop.
    //
    // This is the state before the `EventLoop::run` begins.
    Owned(winit::event_loop::EventLoop<()>),
    // A pointer to the target for building windows.
    //
    // This is the state during `EventLoop::run`. This pointer becomes invalid following
    // `EventLoop::run`, so it is essential to take care that we are in the correct state when
    // using this pointer.
    Pointer(*const winit::event_loop::EventLoopWindowTarget<()>),
}

impl EventLoopWindowTarget {
    // Take a reference to the inner event loop window target.
    //
    // This method is solely used during `window::Builder::build` to allow for
    pub(crate) fn as_ref(&self) -> &winit::event_loop::EventLoopWindowTarget<()> {
        match *self {
            EventLoopWindowTarget::Owned(ref event_loop) => (&**event_loop),
            EventLoopWindowTarget::Pointer(ptr) => {
                // This cast is safe, assuming that the `App`'s `EventLoopWindowTarget` will only
                // ever be in the `Pointer` state while the pointer is valid - that is, during the
                // call to `EventLoop::run`. Great care is taken to ensure that the
                // `EventLoopWindowTarget` is dropped immediately after `EventLoop::run` completes.
                // This allows us to take care of abiding by the `EventLoopWindowTarget` lifetime
                // manually while avoiding having the lifetime propagate up through the `App` type.
                unsafe { &*ptr as &winit::event_loop::EventLoopWindowTarget<()> }
            }
        }
    }
}

// Application Loop.
//
// Beyond this point lies the master function for running the main application loop!
//
// This is undoubtedly the hairiest part of nannou's code base. This is largely due to the fact
// that it is the part of nannou where we marry application and user input events, loop timing,
// updating the model, platform-specific quirks and warts, the various possible `LoopMode`s and
// wgpu interop.
//
// If you would like to contribute but are unsure about any of the following, feel free to open an
// issue and ask!
fn run_loop<M, E>(
    mut app: App,
    model: M,
    event_fn: Option<EventFn<M, E>>,
    update_fn: Option<UpdateFn<M>>,
    default_view: Option<View<M>>,
    exit_fn: Option<ExitFn<M>>,
) where
    M: 'static,
    E: LoopEvent,
{
    // Track the moment the loop starts.
    let loop_start = Instant::now();

    // Wrap the `model` in an `Option`, allowing us to take full ownership within the `event_loop`
    // on `exit`.
    let mut model = Some(model);

    // Take ownership of the `EventLoop` from the `App`.
    let event_loop = match app.event_loop_window_target.take() {
        Some(EventLoopWindowTarget::Owned(event_loop)) => event_loop,
        _ => unreachable!("the app should always own the event loop at this point"),
    };

    // Keep track of state related to the loop mode itself.
    let mut loop_state = LoopState {
        updates_since_event: 0,
        loop_start,
        last_update: loop_start,
        total_updates: 0,
    };

    // Run the event loop.
    event_loop.run(move |mut event, event_loop_window_target, control_flow| {
        // Set the event loop window target pointer to allow for building windows.
        app.event_loop_window_target = Some(EventLoopWindowTarget::Pointer(
            event_loop_window_target as *const _,
        ));

        let mut exit = false;

        match event {
            // Check to see if we need to emit an update and request a redraw.
            winit::event::Event::MainEventsCleared => {
                if let Some(model) = model.as_mut() {
                    let loop_mode = app.loop_mode();
                    let now = Instant::now();
                    let mut do_update = |loop_state: &mut LoopState| {
                        apply_update(&mut app, model, event_fn, update_fn, loop_state, now);
                    };
                    match loop_mode {
                        LoopMode::NTimes { number_of_updates }
                            if loop_state.total_updates >= number_of_updates as u64 => {}
                        _ => do_update(&mut loop_state),
                    }
                }
            }

            // Request a frame from the user for the specified window.
            //
            // TODO: Only request a frame from the user if this redraw was requested following an
            // update. Otherwise, just use the existing intermediary frame.
            winit::event::Event::RedrawRequested(window_id) => {
                // Take the render data and swapchain.
                // We'll replace them before the end of this block.
                let (mut swap_chain, nth_frame) = {
                    let mut windows = app.windows.borrow_mut();

                    let window = windows
                        .get_mut(&window_id)
                        .expect("no window for redraw request ID");
                    let swap_chain = window
                        .swap_chain
                        .swap_chain
                        .take()
                        .expect("missing swap chain");
                    let nth_frame = window.frame_count;
                    window.frame_count += 1;
                    (swap_chain, nth_frame)
                };

                if let Some(model) = model.as_ref() {
                    let swap_chain_output = swap_chain.get_next_texture();
                    let swap_chain_texture = &swap_chain_output.view;

                    // Borrow the window now that we don't need it mutably until setting the render
                    // data back.
                    let windows = app.windows.borrow();
                    let window = windows
                        .get(&window_id)
                        .expect("failed to find window for redraw request");
                    let frame_data = &window.frame_data;

                    // Construct and emit a frame via `view` for receiving the user's graphics commands.
                    let sf = window.tracked_state.scale_factor;
                    let (w, h) = window
                        .tracked_state
                        .physical_size
                        .to_logical::<f32>(sf)
                        .into();
                    let window_rect = geom::Rect::from_w_h(w, h);
                    let raw_frame = RawFrame::new_empty(
                        window.swap_chain_device_queue_pair().clone(),
                        window_id,
                        nth_frame,
                        swap_chain_texture,
                        window.swap_chain.descriptor.format,
                        window_rect,
                    );

                    // If the user specified a view function specifically for this window, use it.
                    // Otherwise, use the fallback, default view passed to the app if there was one.
                    let window_view = window.user_functions.view.clone();

                    match window_view {
                        Some(window::View::Sketch(view)) => {
                            let data = frame_data.as_ref().expect("missing `frame_data`");
                            let frame = Frame::new_empty(raw_frame, &data.render, &data.capture);
                            view(&app, frame);
                        }
                        Some(window::View::WithModel(view)) => {
                            let data = frame_data.as_ref().expect("missing `frame_data`");
                            let frame = Frame::new_empty(raw_frame, &data.render, &data.capture);
                            let view = view
                                .to_fn_ptr::<M>()
                                .expect("unexpected model argument given to window view function");
                            (*view)(&app, &model, frame);
                        }
                        Some(window::View::WithModelRaw(raw_view)) => {
                            let raw_view = raw_view.to_fn_ptr::<M>().expect(
                                "unexpected model argument given to window raw_view function",
                            );
                            (*raw_view)(&app, &model, raw_frame);
                        }
                        None => match default_view {
                            Some(View::Sketch(view)) => {
                                let data = frame_data.as_ref().expect("missing `frame_data`");
                                let frame =
                                    Frame::new_empty(raw_frame, &data.render, &data.capture);
                                view(&app, frame);
                            }
                            Some(View::WithModel(view)) => {
                                let data = frame_data.as_ref().expect("missing `frame_data`");
                                let frame =
                                    Frame::new_empty(raw_frame, &data.render, &data.capture);
                                view(&app, &model, frame);
                            }
                            None => raw_frame.submit(),
                        },
                    }
                }

                // Replace the render data and swap chain.
                let mut windows = app.windows.borrow_mut();
                let window = windows
                    .get_mut(&window_id)
                    .expect("no window for redraw request ID");

                window.swap_chain.swap_chain = Some(swap_chain);
            }

            // Clear any inactive adapters and devices and poll those remaining.
            winit::event::Event::RedrawEventsCleared => {
                app.wgpu_adapters().clear_inactive_adapters_and_devices();
                // TODO: This seems to cause some glitching and slows down macOS drastically.
                // While not necessary, this would be nice to have to automatically process async
                // read/write callbacks submitted by users who aren't aware that they need to poll
                // their devices in order to make them do work. Perhaps as a workaround we could
                // only poll devices that aren't already associated with a window?
                //app.wgpu_adapters().poll_all_devices(false);
            }

            // Ignore wake-up events for now. Currently, these can only be triggered via the app proxy.
            winit::event::Event::NewEvents(_) => {}

            // Track the number of updates since the last I/O event.
            // This is necessary for the `Wait` loop mode to behave correctly.
            ref _other_event => {
                loop_state.updates_since_event = 0;
            }
        }

        // We must re-build the swap chain if the window was resized.
        if let winit::event::Event::WindowEvent {
            ref mut event,
            window_id,
        } = event
        {
            match event {
                winit::event::WindowEvent::Resized(new_inner_size) => {
                    let mut windows = app.windows.borrow_mut();
                    if let Some(window) = windows.get_mut(&window_id) {
                        window.tracked_state.physical_size = new_inner_size.clone();
                        window.rebuild_swap_chain(new_inner_size.clone().into());
                    }
                }

                winit::event::WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    let mut windows = app.windows.borrow_mut();
                    if let Some(window) = windows.get_mut(&window_id) {
                        window.tracked_state.scale_factor = *scale_factor;
                        window.rebuild_swap_chain(new_inner_size.clone().into());
                    }
                }

                _ => (),
            }
        }

        // Process the event with the users functions and see if we need to exit.
        if let Some(model) = model.as_mut() {
            exit |= process_and_emit_winit_event::<M, E>(&mut app, model, event_fn, &event);
        }

        // Set the control flow based on the loop mode.
        let loop_mode = app.loop_mode();
        *control_flow = match loop_mode {
            LoopMode::Wait => {
                // Trigger some extra updates for conrod GUIs to finish "animating". The number of
                // updates used to be configurable, but I don't think there's any use besides GUI.
                if loop_state.updates_since_event < LoopMode::UPDATES_PER_WAIT_EVENT as usize {
                    let ten_ms = std::time::Instant::now() + std::time::Duration::from_millis(10);
                    ControlFlow::WaitUntil(ten_ms)
                } else {
                    ControlFlow::Wait
                }
            }
            LoopMode::NTimes { number_of_updates }
                if loop_state.total_updates >= number_of_updates as u64 =>
            {
                ControlFlow::Wait
            }
            _ => ControlFlow::Poll,
        };

        // If we need to exit, call the user's function and update control flow.
        if exit {
            if let Some(model) = model.take() {
                if let Some(exit_fn) = exit_fn {
                    exit_fn(&app, model);
                }
            }
            *control_flow = ControlFlow::Exit;
            return;
        }
    });

    // Ensure the app no longer points to the window target now that `run` has completed.
    // TODO: Right now `event_loop.run` can't return. This is just a reminder in case one day the
    // API is changed so that it does return.
    #[allow(unreachable_code)]
    {
        app.event_loop_window_target.take();
    }
}

// Apply an update to the model via the user's function and update the app and loop state
// accordingly.
fn apply_update<M, E>(
    app: &mut App,
    model: &mut M,
    event_fn: Option<EventFn<M, E>>,
    update_fn: Option<UpdateFn<M>>,
    loop_state: &mut LoopState,
    now: Instant,
) where
    M: 'static,
    E: LoopEvent,
{
    // Update the app's durations.
    let since_last = now.duration_since(loop_state.last_update);
    let since_start = now.duration_since(loop_state.loop_start);
    app.duration.since_prev_update = since_last;
    app.duration.since_start = since_start;
    app.time = since_start.secs() as _;
    let update = crate::event::Update {
        since_start,
        since_last,
    };
    // User event function.
    if let Some(event_fn) = event_fn {
        let event = E::from(update.clone());
        event_fn(app, model, event);
    }
    // User update function.
    if let Some(update_fn) = update_fn {
        update_fn(app, model, update);
    }
    loop_state.last_update = now;
    loop_state.total_updates += 1;
    loop_state.updates_since_event += 1;
    // Request redraw from windows.
    let windows = app.windows.borrow();
    for window in windows.values() {
        window.window.request_redraw();
    }
}

// Whether or not the given event should toggle fullscreen.
fn should_toggle_fullscreen(
    winit_event: &winit::event::WindowEvent,
    mods: &winit::event::ModifiersState,
) -> bool {
    let input = match *winit_event {
        winit::event::WindowEvent::KeyboardInput { ref input, .. } => match input.state {
            event::ElementState::Pressed => input,
            _ => return false,
        },
        _ => return false,
    };

    let key = match input.virtual_keycode {
        None => return false,
        Some(k) => k,
    };

    // On linux, check for the F11 key (with no modifiers down).
    //
    // TODO: Somehow add special case for KDE?
    if cfg!(target_os = "linux") {
        if *mods == winit::event::ModifiersState::empty() {
            if let Key::F11 = key {
                return true;
            }
        }

    // On macos and windows check for the logo key plus `f` with no other modifiers.
    } else if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
        if mods.logo() {
            if let Key::F = key {
                return true;
            }
        }
    }

    false
}

// Event handling boilerplate shared between the loop modes.
//
// 1. Checks for exit on escape.
// 2. Removes closed windows from app.
// 3. Emits event via `event_fn`.
// 4. Returns whether or not we should break from the loop.
fn process_and_emit_winit_event<'a, M, E>(
    app: &mut App,
    model: &mut M,
    event_fn: Option<EventFn<M, E>>,
    winit_event: &winit::event::Event<'a, ()>,
) -> bool
where
    M: 'static,
    E: LoopEvent,
{
    // Inspect the event to see if it would require closing the App.
    let mut exit_on_escape = false;
    let mut removed_window = None;
    if let winit::event::Event::WindowEvent {
        window_id,
        ref event,
    } = *winit_event
    {
        // If we should exit the app on escape, check for the escape key.
        if app.exit_on_escape() {
            if let winit::event::WindowEvent::KeyboardInput { input, .. } = *event {
                if let Some(Key::Escape) = input.virtual_keycode {
                    exit_on_escape = true;
                }
            }
        }

        // When a window has been closed, this function is called to remove any state associated
        // with that window so that the state doesn't leak.
        //
        // Returns the `Window` that was removed.
        fn remove_related_window_state(app: &App, window_id: &window::Id) -> Option<Window> {
            app.draw_state.renderers.borrow_mut().remove(window_id);
            app.windows.borrow_mut().remove(window_id)
        }

        if let winit::event::WindowEvent::Destroyed = *event {
            removed_window = remove_related_window_state(app, &window_id);
        // TODO: We should allow the user to handle this case. E.g. allow for doing things like
        // "would you like to save". We currently do this with the app exit function, but maybe a
        // window `close` function would be useful?
        } else if let winit::event::WindowEvent::CloseRequested = *event {
            removed_window = remove_related_window_state(app, &window_id);
        } else {
            // Get the size of the window for translating coords and dimensions.
            let (win_w, win_h, scale_factor) = match app.window(window_id) {
                Some(win) => {
                    // If we should toggle fullscreen for this window, do so.
                    if app.fullscreen_on_shortcut() {
                        if should_toggle_fullscreen(event, &app.keys.mods) {
                            if win.is_fullscreen() {
                                win.set_fullscreen(None);
                            } else {
                                let monitor = win.current_monitor();
                                let fullscreen = winit::window::Fullscreen::Borderless(monitor);
                                win.set_fullscreen(Some(fullscreen));
                            }
                        }
                    }

                    let sf = win.tracked_state.scale_factor;
                    let (w, h) = win.tracked_state.physical_size.to_logical::<f32>(sf).into();
                    (w, h, sf)
                }
                None => (0.0, 0.0, 1.0),
            };

            // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
            let tx = |x: geom::scalar::Default| x - win_w as geom::scalar::Default / 2.0;
            let ty = |y: geom::scalar::Default| -(y - win_h as geom::scalar::Default / 2.0);

            // If the window ID has changed, ensure the dimensions are up to date.
            if *app.focused_window.borrow() != Some(window_id) {
                if app.window(window_id).is_some() {
                    *app.focused_window.borrow_mut() = Some(window_id);
                }
            }

            // Check for events that would update either mouse, keyboard or window state.
            match *event {
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let (x, y) = position.to_logical::<f32>(scale_factor).into();
                    let x = tx(x);
                    let y = ty(y);
                    app.mouse.x = x;
                    app.mouse.y = y;
                    app.mouse.window = Some(window_id);
                }

                winit::event::WindowEvent::MouseInput { state, button, .. } => {
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

                winit::event::WindowEvent::KeyboardInput { input, .. } => {
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
                if let Some(input) = ui::winit_window_event_to_input(event, window) {
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

    // Update the modifier keys within the app if necessary.
    if let winit::event::Event::DeviceEvent { event, .. } = winit_event {
        if let winit::event::DeviceEvent::ModifiersChanged(new_mods) = event {
            app.keys.mods = new_mods.clone();
        }
    }

    // If the user provided an event function and winit::event::Event could be interpreted as some event
    // `E`, use it to update the model.
    if let Some(event_fn) = event_fn {
        if let Some(event) = E::from_winit_event(winit_event, app) {
            event_fn(&app, model, event);
        }
    }

    // If the event was a window event, and the user specified an event function for this window,
    // call it.
    if let winit::event::Event::WindowEvent {
        window_id,
        ref event,
    } = *winit_event
    {
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
            (*raw_window_event_fn)(&app, model, event);
        }

        let (win_w, win_h, scale_factor) = {
            let windows = app.windows.borrow();
            windows
                .get(&window_id)
                .map(|w| {
                    let sf = w.tracked_state.scale_factor;
                    let (w, h) = w.tracked_state.physical_size.to_logical::<f64>(sf).into();
                    (w, h, sf)
                })
                .unwrap_or((0.0, 0.0, 1.0))
        };

        // If the event can be represented by a simplified nannou event, check for relevant user
        // functions to be called.
        if let Some(simple) =
            event::WindowEvent::from_winit_window_event(event, win_w, win_h, scale_factor)
        {
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
                (*window_event_fn)(&app, model, simple.clone());
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
                        (*event_fn)(&app, model, $($arg),*);
                    }
                }};
            }

            // Check for more specific event functions.
            match simple {
                event::WindowEvent::KeyPressed(key) => call_user_function!(key_pressed, key),
                event::WindowEvent::KeyReleased(key) => call_user_function!(key_released, key),
                event::WindowEvent::MouseMoved(pos) => call_user_function!(mouse_moved, pos),
                event::WindowEvent::MousePressed(button) => {
                    call_user_function!(mouse_pressed, button)
                }
                event::WindowEvent::MouseReleased(button) => {
                    call_user_function!(mouse_released, button)
                }
                event::WindowEvent::MouseEntered => call_user_function!(mouse_entered),
                event::WindowEvent::MouseExited => call_user_function!(mouse_exited),
                event::WindowEvent::MouseWheel(amount, phase) => {
                    call_user_function!(mouse_wheel, amount, phase)
                }
                event::WindowEvent::Moved(pos) => call_user_function!(moved, pos),
                event::WindowEvent::Resized(size) => call_user_function!(resized, size),
                event::WindowEvent::Touch(touch) => call_user_function!(touch, touch),
                event::WindowEvent::TouchPressure(pressure) => {
                    call_user_function!(touchpad_pressure, pressure)
                }
                event::WindowEvent::HoveredFile(path) => call_user_function!(hovered_file, path),
                event::WindowEvent::HoveredFileCancelled => {
                    call_user_function!(hovered_file_cancelled)
                }
                event::WindowEvent::DroppedFile(path) => call_user_function!(dropped_file, path),
                event::WindowEvent::Focused => call_user_function!(focused),
                event::WindowEvent::Unfocused => call_user_function!(unfocused),
                event::WindowEvent::Closed => call_user_function!(closed),
            }
        }
    }

    // If the loop was destroyed, we'll need to exit.
    let loop_destroyed = match winit_event {
        winit::event::Event::LoopDestroyed => true,
        _ => false,
    };

    // If any exist conditions were triggered, indicate so.
    let exit = if loop_destroyed || exit_on_escape || app.windows.borrow().is_empty() {
        true
    } else {
        false
    };

    exit
}
