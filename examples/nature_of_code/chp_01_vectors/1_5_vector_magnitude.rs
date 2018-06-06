// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-5: Vector Magnitude
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::view(view);
}

fn view(app: &App, frame: Frame) -> Frame {
    app.main_window().set_inner_size_points(640.0, 360.0);

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    let mut mouse = Vector2::new(app.mouse.x, app.mouse.y);
    let center = Vector2::new(0.0, 0.0);
    mouse -= center;

    let m = mouse.magnitude();

    draw.rect()
        .xy(app.window_rect().top_left())
        .w_h(m, 10.0)
        .rgb(0.0, 0.0, 0.0);

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
