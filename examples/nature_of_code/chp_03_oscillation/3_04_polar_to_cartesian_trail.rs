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
    let rect = Rect::from_w_h(800.0, 200.0);
    let _window = app
        .new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .build()
        .unwrap();

    let r = rect.h() * 0.45;
    let theta = 0.0;

    Model { r, theta }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        // Increase the angle over time
        model.theta += 0.02;
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();

    draw.rect()
        .wh(app.window_rect().wh())
        .rgba(1.0, 1.0, 1.0, 0.03);

    let x = model.r * model.theta.cos();
    let y = model.r * -model.theta.sin();

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(pt2(x, y))
        .rgb(0.5, 0.5, 0.5);

    // Draw an ellipse at cartesian coordinate
    draw.ellipse().x_y(x, y).w_h(48.0, 48.0).rgb(0.5, 0.5, 0.5);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
