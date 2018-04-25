// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-6: Vector Normalize
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
}

fn model(app: &App) -> Model {
    let window = app.new_window().with_dimensions(640, 360).build().unwrap();
    Model { window }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    // A vector that points to the mouse position
    let mut mouse = Vector2::new(app.mouse.x, app.mouse.y);
    // A vector that points to the center of the window
    let center = Vector2::new(0.0,0.0);
    // Subtract center from mouse which results in a vector that points from center to mouse
    mouse -= center;

    // Normalize the vector
    mouse = mouse.normalize();
        
    // Multiply its length by 150
    mouse *= 150.0;

    draw.line()
        .start(Point2::new(0.0, 0.0))
        .end(Point2::new(mouse.x, mouse.y))
        .thickness(2.0)
        .rgb(0.0, 0.0, 0.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
