use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: &Frame) {
    // Prepare to draw.
    let draw = app.draw();

    // Clear the background to purple.
    draw.background().color(PLUM);

    // Draw a blue ellipse with default size and position.
    draw.ellipse().color(STEELBLUE);

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}
