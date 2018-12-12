// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 5-6: Simple Harmonic Motion
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    angle: f32,
    a_velocity: f32,
}

fn model(app: &App) -> Model {
    let angle = 0.0;
    let a_velocity = 0.03;

    app.new_window().with_dimensions(640, 360).view(view).build().unwrap();
    Model { angle, a_velocity }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.angle += model.a_velocity;
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let amplitude = 300.0;
    let x = amplitude * model.angle.sin();

    draw.ellipse()
        .x_y(x as f32, 0.0)
        .w_h(50.0, 50.0)
        .rgba(0.5, 0.5, 0.5, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
