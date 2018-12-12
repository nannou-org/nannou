extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: WindowId,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let _window = app.new_window()
        .with_dimensions(512, 512)
        .with_title("nannou")
        .view(view) // The function that will be called for presenting graphics to a frame.
        .event(event) // The function that will be called when the window receives events.
        .build()
        .unwrap();
    Model { _window }
}

// Handle events related to the window and update the model if necessary
fn event(_app: &App, _model: &mut Model, event: WindowEvent) {
    println!("{:?}", event);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    // Clear the window with a "dark charcoal" shade.
    frame.clear(BLUE);
    // Return the cleared frame.
    frame
}
