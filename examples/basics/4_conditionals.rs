extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::view(view);
}

fn view(app: &App, frame: Frame) -> Frame {
    // Prepare to draw.
    let draw = app.draw();

    draw.background().color(LIGHT_PURPLE);

    if app.mouse.x < 0.0 {
        draw.ellipse().color(DARK_BLUE);
    } else {
        draw.ellipse().color(DARK_GREEN);
    }

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
