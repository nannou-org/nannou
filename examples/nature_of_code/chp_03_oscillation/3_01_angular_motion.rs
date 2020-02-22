// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 3-01: Angular Motion
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    angle: f32,
    a_velocity: f32,
    a_acceleration: f32,
}

fn model(app: &App) -> Model {
    let angle = 0.0;
    let a_velocity = 0.0;
    let a_acceleration = 0.0001;
    app.new_window().size(800, 200).view(view).build().unwrap();
    Model {
        angle,
        a_velocity,
        a_acceleration,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.angle += model.a_velocity;
    model.a_velocity += model.a_acceleration;
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    draw.line()
        .start(pt2(-60.0, 0.0))
        .end(pt2(60.0, 0.0))
        .color(BLACK)
        .rotate(model.angle);

    draw.ellipse()
        .x_y(60.0, 0.0)
        .w_h(16.0, 16.0)
        .color(BLACK)
        .rotate(model.angle);

    draw.ellipse()
        .x_y(-60.0, 0.0)
        .w_h(16.0, 16.0)
        .color(BLACK)
        .rotate(model.angle);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
