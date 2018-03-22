extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model {}

fn model(_app: &App) -> Model {
    Model {}
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent { simple: Some(event), .. } => match event {

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

fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    // Color the window with a "dark charcoal" color.
    frame.clear_all(DARK_CHARCOAL);
    // Return the drawn frame.
    frame
}
