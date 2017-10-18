extern crate nannou;

use nannou::{App, Event, Frame};
use nannou::window;
use nannou::event::SimpleWindowEvent::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    window: window::Id,
}

fn model(app: &App) -> Model {
    let window = app.new_window().build().unwrap();
    Model { window }
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent(_id, event) => match event.simple {

            Moved(_pos) => {
            },

            KeyPressed(_key) => {
            },

            KeyReleased(_key) => {
            },

            MouseMoved(_pos) => {
            },

            MouseDragged(_pos, _button) => {
            },

            MousePressed(_button) => {
            },

            MouseReleased(_button) => {
            },

            MouseEntered => {
            },

            MouseExited => {
            },

            Resized(_size) => {
            },

            _other => (),
        },

        Event::Update(_dt) => {
        },

        _ => (),
    }
    model
}

fn view(_app: &App, model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it gray.
    frame.window(model.window).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);
    // Return the drawn frame.
    frame
}
