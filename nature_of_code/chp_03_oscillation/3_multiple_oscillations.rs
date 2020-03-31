// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-x: Multiple Oscillations
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    angle1: f32,
    a_velocity1: f32,
    amplitude1: f32,
    angle2: f32,
    a_velocity2: f32,
    amplitude2: f32,
}

fn model(app: &App) -> Model {
    let angle1 = 0.0;
    let a_velocity1 = 0.01;
    let amplitude1 = 300.0;

    let angle2 = 0.0;
    let a_velocity2 = 0.3;
    let amplitude2 = 10.0;

    app.new_window().size(640, 360).view(view).build().unwrap();
    Model {
        angle1,
        a_velocity1,
        amplitude1,
        angle2,
        a_velocity2,
        amplitude2,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.angle1 += model.a_velocity1;
    model.angle2 += model.a_velocity2;
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let mut x = 0.0;
    x += model.amplitude1 * model.angle1.cos();
    x += model.amplitude2 * model.angle2.sin();

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(pt2(x, 0.0))
        .rgb(0.0, 0.0, 0.0);

    draw.ellipse()
        .x_y(x, 0.0)
        .w_h(20.0, 20.0)
        .rgba(0.7, 0.7, 0.7, 1.0)
        .stroke(BLACK);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
