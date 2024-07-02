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
    velocity: Vec2,
}

fn model(app: &App) -> Model {
    let position = pt2(100.0, 100.0);
    let velocity = vec2(2.5, 5.0);

    let _window = app.new_window().size(200, 200).view(view).build();
    Model { position, velocity }
}

fn update(app: &App, m: &mut Model) {
    // Add the current speed to the position.
    m.position += m.velocity;

    let rect = app.window_rect();
    if (m.position.x > rect.right()) || (m.position.x < rect.left()) {
        m.velocity.x *= -1.0;
    }
    if (m.position.y > rect.top()) || (m.position.y < rect.bottom()) {
        m.velocity.y *= -1.0;
    }
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();

    draw.rect()
        .wh(app.window_rect().wh())
        .srgba(1.0, 1.0, 1.0, 0.03);

    // Display circle at x position
    draw.ellipse()
        .x_y(model.position.x, model.position.y)
        .w_h(16.0, 16.0)
        .gray(0.5)
        .stroke(BLACK);
}
