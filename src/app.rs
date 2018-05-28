use audio;
use audio::cpal;
use draw;
use find_folder;
use frame::Frame;
use geom;
use glium::glutin;
use state;
use std;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{self, AtomicBool};
use std::thread;
use std::time::Duration;
use window::{self, Window};
use ui;

/// An **App** represents the entire context of your application.
///
/// The **App** owns and manages:
///
/// - The **window and input event loop** used to drive the application forward.
/// - **All OpenGL windows** for graphics and user input. Windows can be referenced via their IDs.
/// - The **audio event loop** from which you can receive or send audio via streams.
pub struct App {
    pub(crate) events_loop: glutin::EventsLoop,
    pub(crate) windows: RefCell<HashMap<window::Id, Window>>,
    config: RefCell<Config>,
    pub(crate) ui: ui::Arrangement,
    draw_state: DrawState,
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

/// Miscellaneous app configuration parameters.
#[derive(Debug)]
struct Config {
    loop_mode: LoopMode,
    exit_on_escape: bool,
    fullscreen_on_shortcut: bool,
}

/// A `nannou::Draw` instance owned by the `App`.
///
/// This is a conveniently accessible `Draw` instance which can be easily re-used between calls to
/// an app's `view` function.
#[derive(Debug)]
pub struct Draw<'a> {
    window_id: window::Id,
    draw: RefMut<'a, draw::Draw<DrawScalar>>,
    renderer: RefMut<'a, RefCell<draw::backend::glium::Renderer>>,
}

// Draw state managed by the **App**.
#[derive(Debug)]
struct DrawState {
    draw: RefCell<draw::Draw<DrawScalar>>,
    renderer: RefCell<Option<RefCell<draw::backend::glium::Renderer>>>,
}

/// The app uses a set scalar type in order to provide a simplistic API to users.
///
/// If you require changing the scalar type to something else, consider using a custom
/// `nannou::draw::Draw` instance.
pub type DrawScalar = geom::DefaultScalar;

/// An **App**'s audio API.
pub struct Audio {
    event_loop: Arc<cpal::EventLoop>,
    process_fn_tx: RefCell<Option<mpsc::Sender<audio::stream::ProcessFnMsg>>>,
}

/// A handle to the **App** that can be shared across threads.
///
/// This can be used to "wake up" the **App**'s inner event loop.
pub struct Proxy {
    events_loop_proxy: glutin::EventsLoopProxy,
    events_loop_is_asleep: Arc<AtomicBool>,
}

/// The mode in which the **App** is currently running the event loop.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LoopMode {
    /// Specifies that the application is continuously looping at a consistent rate.
    ///
    /// An application running in the **Rate** loop mode will behave as follows:
    ///
    /// 1. Poll for and collect all pending user input.
    ///    `update` is then called with all application events that have occurred.
    ///
    /// 2. `update` is called with an `Event::Update`.
    ///
    /// 3. `draw` is called.
    ///
    /// 4. Check the time and sleep for the remainder of the `update_intervale`.
    Rate {
        /// The minimum interval between emitted updates.
        update_interval: Duration,
    },
    Wait {
        /// The number of `update`s (and in turn `draw`s) that should occur since the application
        /// last received a non-`Update` event.
        updates_following_event: usize,
        /// The minimum interval between emitted updates.
        update_interval: Duration,
    },
}

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
        LoopMode::rate_fps(Self::DEFAULT_RATE_FPS)
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
    pub(super) fn new(events_loop: glutin::EventsLoop) -> Self {
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
        App {
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
        let exe_path = std::env::current_exe()?;
        find_folder::Search::ParentsThenKids(5, 3)
            .of(exe_path
                .parent()
                .expect("executable has no parent directory to search")
                .into())
            .for_folder(Self::ASSETS_DIRECTORY_NAME)
    }

    /// Begin building a new OpenGL window.
    pub fn new_window<'a>(&'a self) -> window::Builder<'a, 'static> {
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
        self.window(self.window_id()).expect("no window for focused id")
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
    pub fn loop_mode(&self) -> LoopMode {
        self.config.borrow().loop_mode
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
        Proxy { events_loop_proxy, events_loop_is_asleep }
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
        let facade = window.inner_glium_display();
        let draw = self.draw_state.draw.borrow_mut();
        draw.reset();
        if self.draw_state.renderer.borrow().is_none() {
            let renderer = draw::backend::glium::Renderer::new(facade)
                .expect("failed to create `Draw` renderer for glium backend");
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
    pub fn wakeup(&self) -> Result<(), glutin::EventsLoopClosed> {
        if self.events_loop_is_asleep.load(atomic::Ordering::Relaxed) {
            self.events_loop_proxy.wakeup()?;
            self.events_loop_is_asleep.store(false, atomic::Ordering::Relaxed);
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
    ) -> Result<(), draw::backend::glium::RendererDrawError> {
        let window = app.window(self.window_id)
            .expect("no window to draw to for `app::Draw`'s window_id");
        let dpi_factor = window.hidpi_factor();
        let facade = window.inner_glium_display();
        let mut renderer = self.renderer.borrow_mut();
        let mut window_frame = frame
            .window(self.window_id)
            .expect("no frame to draw to for `app::Draw`'s window_id");
        renderer.draw(&self.draw, facade, dpi_factor, &mut (**window_frame).frame)
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
