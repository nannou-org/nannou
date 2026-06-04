// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 8-1: Simple Recursion
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model;

fn model(app: &App) -> Model {
    app.set_update_mode(UpdateMode::freeze());
    let _window = app.new_window().size(640, 360).view(view).build();
    Model
}

fn view(app: &App, _model: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    draw_circle(&draw, 0.0, 0.0, app.window_rect().w());
}

fn draw_circle(draw: &Draw, x: f32, y: f32, mut r: f32) {
    draw.ellipse()
        .x_y(x, y)
        .radius(r)
        .hsv(map_range(r, 2.0, 360.0, 0.0, 1.0), 0.75, 1.0)
        .stroke(BLACK);

    // Exit condition, stop when radius is too small
    if r > 2.0 {
        r *= 0.75;
        // Call the function insie the function! (recursion!)
        draw_circle(draw, x, y, r);
    }
}
