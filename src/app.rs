use glium::{self, glutin};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::Duration;
use window;

/// An **App** represents the entire context of your application.
///
/// The **App** owns and manages:
///
/// - the event loop (used to drive the application forward) 
/// - all OpenGL windows (for graphics and user input, can be referenced via IDs).
pub struct App {
    pub(super) events_loop: glutin::EventsLoop,
    pub(crate) displays: RefCell<HashMap<window::Id, glium::Display>>,
    pub(super) exit_on_escape: Cell<bool>,
    loop_mode: Cell<LoopMode>,
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
    pub const DEFAULT_EXIT_ON_ESCAPE: bool = true;

    // Create a new `App`.
    pub(super) fn new(events_loop: glutin::EventsLoop) -> Self {
        let displays = RefCell::new(HashMap::new());
        let exit_on_escape = Cell::new(Self::DEFAULT_EXIT_ON_ESCAPE);
        let loop_mode = Cell::new(LoopMode::default());
        App {
            events_loop,
            displays,
            exit_on_escape,
            loop_mode,
        }
    }

    /// Begin building a new OpenGL window.
    pub fn new_window<'a>(&'a self) -> window::Builder<'a, 'static> {
        window::Builder::new(self)
    }

    /// The number of windows currently in the application.
    pub fn window_count(&self) -> usize {
        self.displays.borrow().len()
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
}

impl Proxy {
    /// Wake up the application!
    ///
    /// This wakes up the **App**'s inner event loop and inserts an **Awakened** event.
    pub fn wakeup(&self) -> Result<(), glutin::EventsLoopClosed> {
        self.events_loop_proxy.wakeup()
    }
}
