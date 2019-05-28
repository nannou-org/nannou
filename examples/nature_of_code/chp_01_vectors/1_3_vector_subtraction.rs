// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-3: Vector Subtraction
use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: Frame) -> Frame {
    app.main_window().set_inner_size_points(640.0, 360.0);

    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let mut mouse = app.mouse.position();
    let center = vec2(0.0, 0.0);
    mouse -= center;

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
