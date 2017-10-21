use App;
use math::{Point2, Vector2};
use glium::glutin;
use std::path::PathBuf;
use std::time::Duration;
use window;

pub use glium::glutin::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, Touch,
                        TouchPhase, VirtualKeyCode as Key};

/// Event types that are compatible with the nannou app loop.
pub trait LoopEvent: From<Update> {
    fn from_glutin_event(glutin::Event, &App) -> Option<Self>;
}

/// Update event
#[derive(Clone, Debug)]
pub struct Update {
    /// The duration since the last update was emitted.
    ///
    /// The first update's delta will be the time since the given `model` function returned.
    pub since_last: Duration,
    /// The duration since the start of the app loop.
    ///
    /// Specifically, this is the duration of time since the given `model` function returned.
    pub since_start: Duration,
}

#[derive(Clone, Debug)]
pub enum Event {
    /// A window-specific event has occurred for the window with the given Id.
    ///
    /// This event is portrayed both in its "raw" form (the **glutin::WindowEvent**) and its
    /// simplified, new-user-friendly form **SimpleWindowEvent**.
    WindowEvent {
        id: window::Id,
        raw: glutin::WindowEvent,
        simple: Option<SimpleWindowEvent>,
    },
    /// A device-specific event has occurred for the device with the given Id.
    DeviceEvent(glutin::DeviceId, glutin::DeviceEvent),
    /// A timed update alongside the duration since the last update was emitted.
    ///
    /// The first update's delta will be the time since the `model` function returned.
    Update(Update),
    /// The application has been awakened.
    Awakened,
    /// The application has been suspended or resumed.
    ///
    /// The parameter is true if app was suspended, and false if it has been resumed.
    Suspended(bool),
}

#[derive(Clone, Debug)]
pub struct WindowEvent {
    /// A simplified, interpreted version of the `raw` `glutin::WindowEvent` emitted via glutin.
    ///
    /// See the [SimpleWindowEvent](./enum.SimpleWindowEvent.html)
    pub simple: Option<SimpleWindowEvent>,
    /// The original event type produced by `glutin`.
    pub raw: glutin::WindowEvent,
}

/// A simplified version of glutin's `WindowEvent` type to make it easier to get started.
///
/// All co-ordinates and dimensions are DPI-agnostic scalar values.
///
/// Co-ordinates for each window are as follows:
///
/// - `(0.0, 0.0)` is the centre of the window.
/// - positive `x` points to the right, negative `x` points to the left.
/// - positive `y` points upwards, negative `y` points downwards.
/// - positive `z` points into the screen, negative `z` points out of the screen.
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleWindowEvent {

    /// The window has been moved to a new position.
    Moved(Point2<f64>),

    /// The given keyboard key was pressed.
    KeyPressed(Key),

    /// The given keyboard key was released.
    KeyReleased(Key),

    /// The mouse moved to the given x, y position.
    MouseMoved(Point2<f64>),

    /// The given mouse button was dragged to the given x, y position.
    MouseDragged(Point2<f64>, MouseButton),

    /// The given mouse button was pressed.
    MousePressed(MouseButton),

    /// The given mouse button was released.
    MouseReleased(MouseButton),

    /// The mouse entered the window.
    MouseEntered,

    /// The mouse exited the window.
    MouseExited,

    /// A mouse wheel movement or touchpad scroll occurred.
    MouseWheel(MouseScrollDelta, TouchPhase),

    /// The window was resized to the given dimensions.
    Resized(Vector2<f64>),

    /// A file at the given path was hovered over the window.
    HoveredFile(PathBuf),

    /// A file at the given path was dropped onto the window.
    DroppedFile(PathBuf),

    /// A file at the given path that was hovered over the window was cancelled.
    HoveredFileCancelled,

    /// Received a touch event.
    Touch {
        phase: TouchPhase,
        position: Point2<f64>,
        id: u64,
    },

    /// Touchpad pressure event.
    ///
    /// At the moment, only supported on Apple forcetouch-capable macbooks.
    /// The parameters are: pressure level (value between 0 and 1 representing how hard the touchpad
    /// is being pressed) and stage (integer representing the click level).
    TouchpadPressure {
        pressure: f32,
        stage: i64,
    },

    /// The window gained or lost focus.
    ///
    /// The parameter is true if the window has gained focus, and false if it has lost focus.
    Focused(bool),

    /// The window was closed and is no longer stored in the `App`.
    Closed,
}

impl SimpleWindowEvent {
    /// Produce a simplified, new-user-friendly version of the given `glutin::WindowEvent`.
    ///
    /// This strips rarely needed technical information from the event type such as information
    /// about the source device, scancodes for keyboard events, etc to make the experience of
    /// pattern matching on window events nicer for new users.
    ///
    /// This also interprets the raw pixel positions and dimensions of the raw event into a
    /// dpi-agnostic scalar value where (0, 0, 0) is the centre of the screen with the `y` axis
    /// increasing in the upwards direction.
    ///
    /// If the user requires this extra information, they should use the `raw` field of the
    /// `WindowEvent` type rather than the `simple` one.
    pub fn from_glutin_window_event(
        event: glutin::WindowEvent,
        dpi_factor: f64,
        win_w: u32,
        win_h: u32,
    ) -> Option<Self>
    {
        use self::SimpleWindowEvent::*;

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        //
        // winit produces input events in pixels, so these positions need to be divided by the
        // width and height of the window in order to be DPI agnostic.
        let tx = |x: f64| (x / dpi_factor) - win_w as f64 / 2.0;
        let ty = |y: f64| -((y / dpi_factor) - win_h as f64 / 2.0);
        let tw = |w: f64| w / dpi_factor;
        let th = |h: f64| h / dpi_factor;

        let event = match event {

            glutin::WindowEvent::Resized(new_w, new_h) => {
                let x = tw(new_w as f64);
                let y = th(new_h as f64);
                Resized(Vector2 { x, y })
            },

            glutin::WindowEvent::Moved(x, y) => {
                let x = tx(x as f64);
                let y = ty(y as f64);
                Moved(Point2 { x, y })
            },

            glutin::WindowEvent::Closed => {
                Closed
            },

            glutin::WindowEvent::DroppedFile(path) => {
                DroppedFile(path)
            },

            glutin::WindowEvent::HoveredFile(path) => {
                HoveredFile(path)
            },

            glutin::WindowEvent::HoveredFileCancelled => {
                HoveredFileCancelled
            },

            glutin::WindowEvent::Focused(b) => {
                Focused(b)
            },

            glutin::WindowEvent::MouseMoved { position: (x, y), .. } => {
                let x = tx(x as f64);
                let y = ty(y as f64);
                MouseMoved(Point2 { x, y })
            },

            glutin::WindowEvent::MouseEntered { .. } => {
                MouseEntered
            },

            glutin::WindowEvent::MouseLeft { .. } => {
                MouseExited
            },

            glutin::WindowEvent::MouseWheel { delta, phase, .. } => {
                MouseWheel(delta, phase)
            },

            glutin::WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => MousePressed(button),
                ElementState::Released => MouseReleased(button),
            },

            glutin::WindowEvent::Touch(glutin::Touch { phase, location: (x, y), id, .. }) => {
                let x = tx(x);
                let y = ty(y);
                let position = Point2 { x, y };
                Touch { phase, position, id }
            },

            glutin::WindowEvent::TouchpadPressure { pressure, stage, .. } => {
                TouchpadPressure { pressure, stage }
            },

            glutin::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(key) => match input.state {
                    ElementState::Pressed => KeyPressed(key),
                    ElementState::Released => KeyReleased(key),
                },
                None => return None,
            },

            glutin::WindowEvent::AxisMotion { .. } |
            glutin::WindowEvent::Refresh |
            glutin::WindowEvent::ReceivedCharacter(_) |
            glutin::WindowEvent::Suspended(_) => {
                return None;
            },
        };

        Some(event)
    }
}

impl LoopEvent for Event {
    /// Convert the given `glutin::Event` to a nannou `Event`.
    fn from_glutin_event(event: glutin::Event, app: &App) -> Option<Self> {
        let event = match event {
            glutin::Event::WindowEvent { window_id, event } => {
                let displays = app.displays.borrow();
                let (dpi_factor, win_w, win_h) = match displays.get(&window_id) {
                    None => (1.0, 0, 0), // The window was likely closed, these will be ignored.
                    Some(display) => {
                        let window = display.gl_window();
                        let dpi_factor = window.hidpi_factor() as f64;
                        match window.get_inner_size() {
                            None => (dpi_factor, 0, 0),
                            Some((w, h)) => (dpi_factor, w, h),
                        }
                    },
                };
                let raw = event.clone();
                let simple = SimpleWindowEvent::from_glutin_window_event(event, dpi_factor, win_w, win_h);
                Event::WindowEvent { id: window_id, raw, simple }
            },
            glutin::Event::DeviceEvent { device_id, event } =>
                Event::DeviceEvent(device_id, event),
            glutin::Event::Awakened =>
                Event::Awakened,
            // glutin::Event::Suspended(b) =>
            //     Event::Suspended(b),
        };
        Some(event)
    }
}

impl From<Update> for Event {
    fn from(update: Update) -> Self {
        Event::Update(update)
    }
}
