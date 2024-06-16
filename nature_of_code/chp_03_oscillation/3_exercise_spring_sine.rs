// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    angle: f32,
    a_velocity: f32,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build();
    Model {
        angle: 0.0,
        a_velocity: 0.05,
    }
}

fn update(_app: &App, model: &mut Model) {
    model.angle += model.a_velocity;
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);
    let win = app.window_rect();
    let y = map_range(
        model.angle.sin(),
        -1.0,
        1.0,
        win.top() - 50.0,
        win.top() - 250.0,
    );

    draw.line()
        .start(pt2(0.0, win.top()))
        .end(pt2(0.0, y))
        .color(BLACK);

    draw.ellipse()
        .x_y(0.0, y)
        .radius(10.0)
        .color(LIGHT_GRAY)
        .stroke(BLACK);
}
