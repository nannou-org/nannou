extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
}

fn model(app: &App) -> Model {

    let window = app.new_window().with_dimensions(720,720).build().unwrap();
    Model {window}
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

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Prepare to draw.
    let draw = app.draw();

    // Clear the background to pink.
    draw.background().color(LIGHT_PURPLE);

    // Draw a red ellipse with default size and position.
    draw.ellipse().color(DARK_BLUE);

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}

