// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-8: Static Wave Lines
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    angle: f32,
    angle_vel: f32,
    vertices: Vec<Point2>,
}

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::loop_once());
    app.new_window().size(640, 360).view(view).build().unwrap();

    Model {
        angle: 0.0,
        angle_vel: 0.1,
        vertices: Vec::new(),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let win = app.window_rect();
    model.vertices.clear();

    for x in (0..(win.w() as usize)).step_by(5) {
        let y = map_range(model.angle.sin(), -1.0, 1.0, win.bottom(), win.top());
        model.angle += model.angle_vel;
        model.vertices.push(pt2(win.left() + x as f32, y as f32));
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    draw.polyline()
        .weight(2.0)
        .points(model.vertices.clone())
        .color(BLACK);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
