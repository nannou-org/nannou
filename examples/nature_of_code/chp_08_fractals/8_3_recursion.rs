// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 8-3: Simple Recursion
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model;

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::loop_once());
    let _window = app.new_window().size(640, 360).view(view).build().unwrap();
    Model
}

fn view(app: &App, _model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    draw_circle(&draw, 0.0, 0.0, 200.0);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

// Recursive function
fn draw_circle(draw: &app::Draw, x: f32, y: f32, r: f32) {
    let norm_radius = map_range(r, 2.0, 360.0, 0.0, 1.0);
    draw.ellipse()
        .x_y(x, y)
        .radius(r)
        .hsva(norm_radius, 0.75, 1.0, norm_radius)
        .stroke(BLACK);

    if r > 8.0 {
        // Four circles! left right, up and down
        draw_circle(&draw, x + r, y, r / 2.0);
        draw_circle(&draw, x - r, y, r / 2.0);
        draw_circle(&draw, x, y + r, r / 2.0);
        draw_circle(&draw, x, y - r, r / 2.0);
    }
}
