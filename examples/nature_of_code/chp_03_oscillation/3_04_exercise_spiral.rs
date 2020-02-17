// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 3-04: Spiral
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    r: f32,
    theta: f32,
}

fn model(app: &App) -> Model {
    let r = 0.0;
    let theta = 0.0;

    app.new_window()
        .dimensions(640, 360)
        .view(view)
        .build()
        .unwrap();
    Model { r, theta }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Increment the angle
    model.theta += 0.01;
    // Increment the radius
    model.r += 0.05;
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    //draw.background().color(WHITE);

    let x = model.r * model.theta.cos();
    let y = model.r * model.theta.sin();

    // Draw an ellipse at x,y
    draw.ellipse()
        .x_y(x, y)
        .w_h(16.0, 16.0)
        .rgba(0.0, 0.0, 0.0, 1.0);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
