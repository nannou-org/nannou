// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-2: Bouncing Ball, with Vector!
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    ball: Ball,
}

struct Ball {
    position: Point2<f32>,
    velocity: Vector2<f32>,
}

impl Ball {
    fn new() -> Self {
        let position = Point2::new(100.0, 100.0);
        let velocity = Vector2::new(2.5, 5.0);
        Ball { position, velocity }
    }

    fn update(&mut self, rect: Rect<f32>) {
        // Add the current speed to the position.
        self.position += self.velocity;

        if (self.position.x > rect.right()) || (self.position.x < rect.left()) {
            self.velocity.x = self.velocity.x * -1.0;
        }
        if (self.position.y > rect.top()) || (self.position.y < rect.bottom()) {
            self.velocity.y = self.velocity.y * -1.0;
        }
    }

    fn display(&self, draw: &app::Draw) {
        // Display circle at x position
        draw.ellipse()
            .x_y(self.position.x, self.position.y)
            .w_h(16.0, 16.0)
            .rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    app.main_window().set_inner_size_points(200.0, 200.0);
    let ball = Ball::new();
    Model { ball }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        m.ball.update(app.window_rect());
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.rect()
        .wh(app.window_rect().wh())
        .rgba(1.0, 1.0, 1.0, 0.03);

    m.ball.display(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
