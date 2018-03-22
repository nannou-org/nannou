extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    window: WindowId,
}

fn model(app: &App) -> Model {
    let window = app.new_window().build().unwrap();
    Model { window }
}

fn event(_app: &App, model: Model, _event: Event) -> Model {
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing 
    let draw = app.draw(model.window).unwrap();

    // Clear the background to blue.
    draw.background().rgb(0.0, 0.0, 1.0);

    // Short-hand helper functions.
    draw.ellipse()
        .x_y(app.mouse.x, app.mouse.y)
        .w_h(app.window.width * 0.5, app.window.height * 0.5)
        .rgb(1.0, 0.0, 0.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
