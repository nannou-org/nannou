// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 3-01: Angular Motion
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).view(view).run();
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

    let _window = app.new_window().with_dimensions(800, 200).build().unwrap();
    Model {
        angle,
        a_velocity,
        a_acceleration,
    }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        model.angle += model.a_velocity;
        model.a_velocity += model.a_acceleration;
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
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

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
