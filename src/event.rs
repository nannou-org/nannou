use glium::glutin;
use std::time::Duration;

pub use glium::glutin::{ElementState, KeyboardInput, WindowEvent, VirtualKeyCode};

pub trait LoopEvent: From<Update> + From<glutin::Event> {}

impl<E> LoopEvent for E where E: From<Update> + From<glutin::Event> {}

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
    WindowEvent(glutin::WindowId, glutin::WindowEvent),
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

impl From<Update> for Event {
    fn from(update: Update) -> Self {
        Event::Update(update)
    }
}

impl From<glutin::Event> for Event {
    fn from(event: glutin::Event) -> Self {
        match event {
            glutin::Event::WindowEvent { window_id, event } =>
                Event::WindowEvent(window_id, event),
            glutin::Event::DeviceEvent { device_id, event } =>
                Event::DeviceEvent(device_id, event),
            glutin::Event::Awakened =>
                Event::Awakened,
            // glutin::Event::Suspended(b) =>
            //     Event::Suspended(b),
        }
    }
}
