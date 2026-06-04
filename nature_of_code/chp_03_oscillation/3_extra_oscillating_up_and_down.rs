// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-9: Wave_A
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    angle: f32,
}

fn model(app: &App) -> Model {
    app.new_window().size(400, 400).view(view).build();
    let angle = 0.0;
    Model { angle }
}

fn update(_app: &App, model: &mut Model) {
    model.angle += 0.02;
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let y = 100.0 * model.angle.sin();

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(pt2(0.0, y))
        .srgb(0.4, 0.4, 0.4);
    draw.ellipse()
        .x_y(0.0, y)
        .w_h(16.0, 16.0)
        .gray(0.4)
        .stroke(BLACK);
}
