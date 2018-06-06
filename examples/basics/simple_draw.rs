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

    // Draw a purple triangle in the top left half of the window.
    let win = app.window_rect();
    draw.tri()
        .points(win.bottom_left(), win.top_left(), win.top_right())
        .color(DARK_PURPLE);

    // Draw an ellipse to follow the mouse.
    let t = app.time;
    draw.ellipse()
        .x_y(app.mouse.x * t.cos(), app.mouse.y)
        .radius(win.w() * 0.125 * t.sin())
        .color(RED);

    // Draw a line!
    draw.line()
        .start(win.top_left() * t.sin())
        .end(win.bottom_right() * t.cos())
        .thickness(win.h() / (50.0 * t.sin()))
        .color(DARK_BLUE);

    // Draw a quad that follows the inverse of the ellipse.
    draw.quad().x_y(-app.mouse.x, app.mouse.y).color(DARK_GREEN).rotate(t);

    // Draw a rect that follows a different inverse of the ellipse.
    draw.rect().x_y(app.mouse.y, app.mouse.x).w(app.mouse.x * 0.25).hsv(t, 1.0, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
