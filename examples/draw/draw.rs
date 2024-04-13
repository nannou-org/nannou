use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(CORNFLOWERBLUE);

    // Draw a purple triangle in the top left half of the window.
    let win = app.window_rect();
    draw.tri()
        .points(win.bottom_left(), win.top_left(), win.top_right())
        .color(VIOLET);

    // Draw an ellipse to follow the mouse.
    let t = app.time().elapsed_seconds();
    draw.ellipse()
        .x_y(app.mouse().x * t.cos(), app.mouse().y)
        .radius(win.w() * 0.125 * t.sin())
        .color(RED);

    // Draw a line!
    draw.line()
        .weight(10.0 + (t.sin() * 0.5 + 0.5) * 90.0)
        .caps_round()
        .color(PALEGOLDENROD)
        .points(win.top_left() * t.sin(), win.bottom_right() * t.cos());

    // Draw a quad that follows the inverse of the ellipse.
    draw.quad()
        .x_y(-app.mouse().x, app.mouse().y)
        .color(DARKGREEN)
        .rotate(t);

    // Draw a rect that follows a different inverse of the ellipse.
    draw.rect()
        .x_y(app.mouse().y, app.mouse().x)
        .w(app.mouse().x * 0.25)
        .hsv(t, 1.0, 1.0);
}
