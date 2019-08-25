// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-2: Bouncing Ball, with Vector!
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    position: Point2,
    velocity: Vector2,
}

fn model(app: &App) -> Model {
    let position = pt2(100.0, 100.0);
    let velocity = vec2(2.5, 5.0);

    let _window = app
        .new_window()
        .with_dimensions(200, 200)
        .view(view)
        .build()
        .unwrap();
    Model { position, velocity }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    // Add the current speed to the position.
    m.position += m.velocity;

    let rect = app.window_rect();
    if (m.position.x > rect.right()) || (m.position.x < rect.left()) {
        m.velocity.x = m.velocity.x * -1.0;
    }
    if (m.position.y > rect.top()) || (m.position.y < rect.bottom()) {
        m.velocity.y = m.velocity.y * -1.0;
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();

    draw.rect()
        .wh(app.window_rect().wh())
        .rgba(1.0, 1.0, 1.0, 0.03);

    // Display circle at x position
    draw.ellipse()
        .x_y(model.position.x, model.position.y)
        .w_h(16.0, 16.0)
        .rgb(0.5, 0.5, 0.5)
        .stroke(BLACK);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
