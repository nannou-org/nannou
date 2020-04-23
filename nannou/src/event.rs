//! Application, event loop and window event definitions and implementations.
//!
//! - [**Event**](./enum.Event.html) - the default application event type.
//! - [**winit::event::WindowEvent**](https://docs.rs/winit/latest/winit/event/enum.WindowEvent.html) -
//!   events related to a single window.
//! - [**WindowEvent**](./enum.WindowEvent.html) - a stripped-back, simplified, newcomer-friendly
//!   version of the **raw**, low-level winit event.

use crate::geom::{self, Point2, Vector2};
use crate::window;
use crate::App;
use std::path::PathBuf;
use winit;

pub use winit::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, TouchPhase,
    VirtualKeyCode as Key,
};

/// Event types that are compatible with the nannou app loop.
pub trait LoopEvent: 'static + From<Update> {
    /// Produce a loop event from the given winit event.
    fn from_winit_event<'a, T>(_: &winit::event::Event<'a, T>, _: &App) -> Option<Self>;
}

/// Update event, emitted on each pass of an application loop.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update {
    /// The duration since the last update was emitted.
    ///
    /// The first update's delta will be the time since the given `model` function returned.
    pub since_last: std::time::Duration,
    /// The duration since the start of the app loop.
    ///
    /// Specifically, this is the duration of time since the given `model` function returned.
    pub since_start: std::time::Duration,
}

/// The default application **Event** type.
#[derive(Debug)]
pub enum Event {
    /// A window-specific event has occurred for the window with the given Id.
    ///
    /// The event is available as a **WindowEvent**, a more user-friendly form of
    /// **winit::event::WindowEvent**. Once
    /// [winit#1387](https://github.com/rust-windowing/winit/issues/1387) is fixed, its "raw" form
    /// will also be available.
    WindowEvent {
        id: window::Id,
        simple: Option<WindowEvent>,
        // TODO: Re-add this when winit#1387 is resolved.
        // raw: winit::event::WindowEvent,
    },

    /// A device-specific event has occurred for the device with the given Id.
    DeviceEvent(winit::event::DeviceId, winit::event::DeviceEvent),

    /// A timed update alongside the duration since the last update was emitted.
    ///
    /// The first update's delta will be the time since the `model` function returned.
    Update(Update),

    /// The application has been suspended or resumed.
    Suspended,
    /// The application has been awakened.
    Resumed,
}

/// The event associated with a touch at a single point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TouchEvent {
    /// The unique ID associated with this touch, e.g. useful for distinguishing between fingers.
    pub id: u64,
    /// The state of the touch.
    pub phase: TouchPhase,
    /// The position of the touch.
    pub position: Point2<geom::scalar::Default>,
}

/// Pressure on a touch pad.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TouchpadPressure {
    /// The unique ID associated with the device that emitted this event.
    pub device_id: winit::event::DeviceId,
    /// The amount of pressure applied.
    pub pressure: f32,
    /// Integer representing the click level.
    pub stage: i64,
}

/// Motion along some axis of a device e.g. joystick or gamepad.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AxisMotion {
    /// The unique ID of the device that emitted this event.
    pub device_id: winit::event::DeviceId,
    /// The axis on which motion occurred.
    pub axis: winit::event::AxisId,
    /// The motion value.
    pub value: geom::scalar::Default,
}

/// A simplified version of winit's `WindowEvent` type to make it easier to get started.
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
pub enum WindowEvent {
    /// The window has been moved to a new position.
    Moved(Point2<geom::scalar::Default>),

    /// The given keyboard key was pressed.
    KeyPressed(Key),

    /// The given keyboard key was released.
    KeyReleased(Key),

    /// The mouse moved to the given x, y position.
    MouseMoved(Point2<geom::scalar::Default>),

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

    /// The window was resized to the given dimensions (in DPI-agnostic points, not pixels).
    Resized(Vector2<geom::scalar::Default>),

    /// A file at the given path was hovered over the window.
    HoveredFile(PathBuf),

    /// A file at the given path was dropped onto the window.
    DroppedFile(PathBuf),

    /// A file at the given path that was hovered over the window was cancelled.
    HoveredFileCancelled,

    /// Received a touch event.
    Touch(TouchEvent),

    /// Touchpad pressure event.
    ///
    /// At the moment, only supported on Apple forcetouch-capable macbooks.
    /// The parameters are: pressure level (value between 0 and 1 representing how hard the touchpad
    /// is being pressed) and stage (integer representing the click level).
    TouchPressure(TouchpadPressure),

    /// The window gained focus.
    Focused,

    /// The window lost focus.
    Unfocused,

    /// The window was closed and is no longer stored in the `App`.
    Closed,
}

impl WindowEvent {
    /// Produce a simplified, new-user-friendly version of the given `winit::event::WindowEvent`.
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
    pub fn from_winit_window_event(
        event: &winit::event::WindowEvent,
        win_w: f64,
        win_h: f64,
        scale_factor: f64,
    ) -> Option<Self> {
        use self::WindowEvent::*;

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        //
        // winit produces input events in pixels, so these positions need to be divided by the
        // width and height of the window in order to be DPI agnostic.
        let tw = |w: f64| w as geom::scalar::Default;
        let th = |h: f64| h as geom::scalar::Default;
        let tx = |x: f64| (x - win_w / 2.0) as geom::scalar::Default;
        let ty = |y: f64| (-(y - win_h / 2.0)) as geom::scalar::Default;

        let event = match event {
            winit::event::WindowEvent::Resized(new_size) => {
                let (new_w, new_h) = new_size.to_logical::<f64>(scale_factor).into();
                let x = tw(new_w);
                let y = th(new_h);
                Resized(Vector2 { x, y })
            }

            winit::event::WindowEvent::Moved(new_pos) => {
                let (new_x, new_y) = new_pos.to_logical::<f64>(scale_factor).into();
                let x = tx(new_x);
                let y = ty(new_y);
                Moved(Point2 { x, y })
            }

            // TODO: Should separate the behaviour of close requested and destroyed.
            winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
                Closed
            }

            winit::event::WindowEvent::DroppedFile(path) => DroppedFile(path.clone()),

            winit::event::WindowEvent::HoveredFile(path) => HoveredFile(path.clone()),

            winit::event::WindowEvent::HoveredFileCancelled => HoveredFileCancelled,

            winit::event::WindowEvent::Focused(b) => {
                if b.clone() {
                    Focused
                } else {
                    Unfocused
                }
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = position.to_logical::<f64>(scale_factor).into();
                let x = tx(x);
                let y = ty(y);
                MouseMoved(Point2 { x, y })
            }

            winit::event::WindowEvent::CursorEntered { .. } => MouseEntered,

            winit::event::WindowEvent::CursorLeft { .. } => MouseExited,

            winit::event::WindowEvent::MouseWheel { delta, phase, .. } => {
                MouseWheel(delta.clone(), phase.clone())
            }

            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => MousePressed(button.clone()),
                ElementState::Released => MouseReleased(button.clone()),
            },

            winit::event::WindowEvent::Touch(winit::event::Touch {
                phase,
                location,
                id,
                ..
            }) => {
                let (x, y) = location.to_logical::<f64>(scale_factor).into();
                let x = tx(x);
                let y = ty(y);
                let position = Point2 { x, y };
                let touch = TouchEvent {
                    phase: phase.clone(),
                    position,
                    id: id.clone(),
                };
                WindowEvent::Touch(touch)
            }

            winit::event::WindowEvent::TouchpadPressure {
                device_id,
                pressure,
                stage,
            } => TouchPressure(TouchpadPressure {
                device_id: device_id.clone(),
                pressure: pressure.clone(),
                stage: stage.clone(),
            }),

            winit::event::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(key) => match input.state {
                    ElementState::Pressed => KeyPressed(key),
                    ElementState::Released => KeyReleased(key),
                },
                None => return None,
            },

            winit::event::WindowEvent::ModifiersChanged(_) => {
                return None;
            }

            winit::event::WindowEvent::AxisMotion { .. }
            | winit::event::WindowEvent::ReceivedCharacter(_)
            | winit::event::WindowEvent::ThemeChanged(_)
            | winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                return None;
            }
        };

        Some(event)
    }
}

impl LoopEvent for Event {
    /// Convert the given `winit::event::Event` to a nannou `Event`.
    fn from_winit_event<'a, T>(event: &winit::event::Event<'a, T>, app: &App) -> Option<Self> {
        let event = match event {
            winit::event::Event::WindowEvent { window_id, event } => {
                let windows = app.windows.borrow();
                let (win_w, win_h, scale_factor) = match windows.get(&window_id) {
                    None => (0.0, 0.0, 1.0), // The window was likely closed, these will be ignored.
                    Some(window) => {
                        let sf = window.tracked_state.scale_factor;
                        let size = window.tracked_state.physical_size;
                        let (w, h) = size.to_logical::<f64>(sf).into();
                        (w, h, sf)
                    }
                };
                let simple =
                    WindowEvent::from_winit_window_event(event, win_w, win_h, scale_factor);
                Event::WindowEvent {
                    id: window_id.clone(),
                    simple,
                    // TODO: Re-add this when winit#1387 is resolved.
                    // raw,
                }
            }
            winit::event::Event::DeviceEvent { device_id, event } => {
                Event::DeviceEvent(device_id.clone(), event.clone())
            }
            winit::event::Event::Suspended => Event::Suspended,
            winit::event::Event::Resumed => Event::Resumed,
            winit::event::Event::NewEvents(_)
            | winit::event::Event::UserEvent(_)
            | winit::event::Event::MainEventsCleared
            | winit::event::Event::RedrawRequested(_)
            | winit::event::Event::RedrawEventsCleared
            | winit::event::Event::LoopDestroyed => return None,
        };
        Some(event)
    }
}

impl From<Update> for Event {
    fn from(update: Update) -> Self {
        Event::Update(update)
    }
}
