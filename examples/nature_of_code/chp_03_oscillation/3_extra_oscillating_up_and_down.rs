// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-9: Wave_A
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).view(view).run();
}

struct Model {
    angle: f32,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().with_dimensions(400, 400).build().unwrap();
    let angle = 0.0;
    Model { angle }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        model.angle += 0.02;
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let y = 100.0 * model.angle.sin();

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(pt2(0.0, y))
        .rgb(0.4, 0.4, 0.4);
    draw.ellipse()
        .x_y(0.0, y)
        .w_h(16.0, 16.0)
        .rgb(0.4, 0.4, 0.4);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
