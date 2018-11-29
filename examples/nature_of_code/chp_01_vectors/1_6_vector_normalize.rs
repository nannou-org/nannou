// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-6: Vector Normalize
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: Frame) -> Frame {
    app.main_window().set_inner_size_points(640.0, 360.0);

    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // A vector that points to the mouse position
    let mut mouse = vec2(app.mouse.x, app.mouse.y);
    // A vector that points to the center of the window
    let center = vec2(0.0, 0.0);
    // Subtract center from mouse which results in a vector that points from center to mouse
    mouse -= center;

    // Normalize the vector
    mouse = mouse.normalize();

    // Multiply its length by 150
    mouse *= 150.0;

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(mouse)
        .thickness(2.0)
        .color(BLACK);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
