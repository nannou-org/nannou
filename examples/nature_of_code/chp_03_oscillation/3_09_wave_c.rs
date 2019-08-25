// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-9: Wave_C
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    start_angle: f32,
    angle_vel: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(200, 200)
        .view(view)
        .build()
        .unwrap();
    let start_angle = 0.0;
    let angle_vel = 0.4;
    Model {
        start_angle,
        angle_vel,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.start_angle += 0.015;
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let mut angle = model.start_angle;
    let rect = app.window_rect();
    let mut x = rect.left();
    while x <= rect.right() {
        let y = map_range(angle.sin(), -1.0, 1.0, rect.top(), rect.bottom());
        draw.ellipse()
            .x_y(x as f32, y)
            .w_h(48.0, 48.0)
            .rgba(0.0, 0.0, 0.0, 0.5)
            .stroke(BLACK)
            .stroke_weight(2.0);

        angle += model.angle_vel;
        x += 24.0;
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
