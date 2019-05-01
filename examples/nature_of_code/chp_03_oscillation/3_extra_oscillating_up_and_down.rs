// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-9: Wave_A
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    angle: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(400, 400)
        .view(view)
        .build()
        .unwrap();
    let angle = 0.0;
    Model { angle }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.angle += 0.02;
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
