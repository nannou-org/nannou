// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 3-04: Spiral
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).view(view).run();
}

struct Model {
    r: f32,
    theta: f32,
}

fn model(app: &App) -> Model {
    let r = 0.0;
    let theta = 0.0;

    let _window = app.new_window().with_dimensions(640, 360).build().unwrap();
    Model { r, theta }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        // Increment the angle
        model.theta += 0.01;
        // Increment the radius
        model.r += 0.05;
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
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

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
