extern crate nannou;

use nannou::{App, Frame};
use nannou::event::{ElementState, Event, Update, WindowEvent};
use nannou::window;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    window: window::Id,
    // Add the state of your application here.
}

fn model(app: &App) -> Model {
    let window = app.new_window().build().unwrap();
    Model { window }
}

fn event(app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent(id, event) => window_event(app, model, id, event),
        Event::Update(dt) => update(app, model, dt),
        _ => model,
    }
}

fn window_event(_app: &App, model: Model, _window_id: window::Id, event: WindowEvent) -> Model {
    match event {
        WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
            Some(_key) => match input.state {
                ElementState::Pressed => {
                    // Handle key presses.
                },
                ElementState::Released => {
                    // Handle key releases.
                },
            },
            _ => (),
        },
        WindowEvent::MouseInput { state, button, .. } => match state {
            ElementState::Pressed => {
                // Handle mouse presses.
            },
            ElementState::Released => {
                // Handle mouse releases.
            },
        },
        WindowEvent::MouseMoved { position, .. } => {
            // Handle mouse movement.
        },
        _ => (),
    }
    model
}

fn update(_app: &App, model: Model, _update: Update) -> Model {
    model
}

fn view(_app: &App, model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it gray.
    frame.window(model.window).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);
    // Return the drawn frame.
    frame
}
