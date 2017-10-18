extern crate nannou;

use nannou::{App, Event, Frame};

fn main() {
    nannou::run(model, update, draw);
}

struct Model {
    window: nannou::window::Id,
}

fn model(app: &App) -> Model {
    let window = app.new_window().build().unwrap();
    Model { window }
}

fn update(_app: &App, model: Model, event: Event) -> Model {
    match event {
        // Handle window events like mouse, keyboard, resize, etc here.
        Event::WindowEvent(_id, event) => {
            println!("{:?}", event.simple);
        },
        // `Update` the model here.
        Event::Update(_update) => {
        },
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn draw(_app: &App, model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it grey.
    frame.window(model.window).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);
    // Return the cleared frame.
    frame
}
