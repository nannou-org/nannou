// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-2: Bouncing Ball, with Vector!
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    ball: Ball,
}

struct Ball {
    position: Point2,
    velocity: Vector2,
}

impl Ball {
    fn new() -> Self {
        let position = pt2(100.0, 100.0);
        let velocity = vec2(2.5, 5.0);
        Ball { position, velocity }
    }

    fn update(&mut self, rect: Rect<f32>) {
        // Add the current speed to the position.
        self.position += self.velocity;

        if self.position.x > rect.right() || self.position.x < rect.left() {
            self.velocity.x = self.velocity.x * -1.0;
        }
        if self.position.y > rect.top() || self.position.y < rect.bottom() {
            self.velocity.y = self.velocity.y * -1.0;
        }
    }

    fn display(&self, draw: &app::Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(16.0, 16.0)
            .rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    app.main_window().set_inner_size_points(300.0, 300.0);
    let ball = Ball::new();
    Model { ball }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.ball.update(app.window_rect());
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.rect()
        .wh(app.window_rect().wh())
        .rgba(1.0, 1.0, 1.0, 0.03);

    m.ball.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
