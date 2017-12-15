use audio;
use audio::cpal;
use find_folder;
use glium::glutin;
use state;
use std;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use window::{self, Window};
use ui;

/// An **App** represents the entire context of your application.
///
/// The **App** owns and manages:
///
/// - the event loop (used to drive the application forward) 
/// - all OpenGL windows (for graphics and user input, can be referenced via IDs).
pub struct App {
    pub(crate) events_loop: glutin::EventsLoop,
    pub(crate) windows: RefCell<HashMap<window::Id, Window>>,
    pub(super) exit_on_escape: Cell<bool>,
    pub(crate) ui: ui::Arrangement,
    loop_mode: Cell<LoopMode>,

    /// The `App`'s audio-related API.
    pub audio: Audio,

    /// The current state of the `Mouse`.
    pub mouse: state::Mouse,
    /// State of the window currently in focus.
    pub window: state::Window,
}

/// An **App**'s audio API.
pub struct Audio {
    event_loop: Arc<cpal::EventLoop>,
    process_fn_tx: RefCell<Option<mpsc::Sender<audio::stream::output::ProcessFnMsg>>>,
}

/// A handle to the **App** that can be shared across threads.
///
/// This can be used to "wake up" the **App**'s inner event loop.
pub struct Proxy {
    events_loop_proxy: glutin::EventsLoopProxy,
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

impl App {
    pub const ASSETS_DIRECTORY_NAME: &'static str = "assets";
    pub const DEFAULT_EXIT_ON_ESCAPE: bool = true;

    // Create a new `App`.
    pub(super) fn new(events_loop: glutin::EventsLoop) -> Self {
        let windows = RefCell::new(HashMap::new());
        let exit_on_escape = Cell::new(Self::DEFAULT_EXIT_ON_ESCAPE);
        let loop_mode = Cell::new(LoopMode::default());
        let cpal_event_loop = Arc::new(cpal::EventLoop::new());
        let process_fn_tx = RefCell::new(None);
        let audio = Audio { event_loop: cpal_event_loop, process_fn_tx };
        let ui = ui::Arrangement::new();
        let mouse = state::Mouse::new();
        let window = state::Window::new();
        App {
            events_loop,
            windows,
            exit_on_escape,
            loop_mode,
            audio,
            ui,
            mouse,
            window,
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
            .of(exe_path.parent().expect("executable has no parent directory to search").into())
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

    /// Return whether or not the `App` is currently set to exit when the `Escape` key is pressed.
    pub fn exit_on_escape(&self) -> bool {
        self.exit_on_escape.get()
    }

    /// Specify whether or not the app should close when the `Escape` key is pressed.
    ///
    /// By default this is `true`.
    pub fn set_exit_on_escape(&self, b: bool) {
        self.exit_on_escape.set(b);
    }

    /// Returns the **App**'s current **LoopMode**.
    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode.get()
    }

    /// Sets the loop mode of the **App**.
    ///
    /// Note: Setting the loop mode will not affect anything until the end of the current loop
    /// iteration. The behaviour of a single loop iteration is described under each of the
    /// **LoopMode** variants.
    pub fn set_loop_mode(&self, mode: LoopMode) {
        self.loop_mode.set(mode);
    }

    /// A handle to the **App** that can be shared across threads.
    ///
    /// This can be used to "wake up" the **App**'s inner event loop.
    pub fn create_proxy(&self) -> Proxy {
        let events_loop_proxy = self.events_loop.create_proxy();
        Proxy { events_loop_proxy }
    }

    /// Create a new `Ui` for the window with the given `Id`.
    ///
    /// Returns `None` if there is no window for the given `window_id`.
    pub fn new_ui(&self, window_id: window::Id) -> ui::Builder {
        ui::Builder::new(self, window_id)
    }
}

impl Audio {
    /// Enumerate the available audio output devices on the system.
    ///
    /// Produces an iterator yielding a `audio::stream::DeviceId` for each output device.
    pub fn output_devices(&self) -> audio::stream::output::Devices {
        let endpoints = cpal::endpoints();
        audio::stream::output::Devices { endpoints }
    }

    /// The current default audio output device.
    pub fn default_output_device(&self) -> Option<audio::stream::output::Device> {
        cpal::default_endpoint()
            .map(|endpoint| audio::stream::output::Device { endpoint })
    }

    /// Begin building a new output audio stream.
    ///
    /// The first time this is called, this method will spawn the `cpal::EventLoop::run` method on
    /// its own thread, ready to run built `Voice`s.
    pub fn new_output_stream<M, F, S>(&self, model: M, render: F)
        -> audio::stream::output::Builder<M, F, S>
    {
        let process_fn_tx = if self.process_fn_tx.borrow().is_none() {
            let event_loop = self.event_loop.clone();
            let (tx, rx) = mpsc::channel();
            let mut loop_context = audio::stream::output::LoopContext::new(rx);
            thread::Builder::new()
                .name("cpal::EventLoop::run thread".into())
                .spawn(move || event_loop.run(move |v_id, out| loop_context.process(v_id, out)))
                .expect("failed to spawn cpal::EventLoop::run thread");
            *self.process_fn_tx.borrow_mut() = Some(tx.clone());
            tx
        } else {
            self.process_fn_tx.borrow().as_ref().unwrap().clone()
        };

        audio::stream::output::Builder {
            event_loop: self.event_loop.clone(),
            process_fn_tx: process_fn_tx,
            model,
            render,
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
    pub fn wakeup(&self) -> Result<(), glutin::EventsLoopClosed> {
        self.events_loop_proxy.wakeup()
    }
}
