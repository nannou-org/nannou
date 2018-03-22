extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::view(view);
}

fn view(app: &App, frame: Frame) -> Frame {
    // Begin drawing 
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(BLUE);

    // Short-hand helper functions.
    draw.ellipse()
        .x_y(app.mouse.x, app.mouse.y)
        .w_h(app.window.width * 0.5, app.window.height * 0.5)
        .color(RED);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
