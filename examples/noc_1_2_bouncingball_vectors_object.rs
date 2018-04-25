// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example x-x: Bouncing Ball, with Vector!
extern crate nannou;

use nannou::prelude::*;
use nannou::app::Draw;
use nannou::geom::rect::Rect;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
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

    fn display(&self, draw: &Draw) {
        // Display circle at x position
        draw.ellipse()
            .x_y(self.position.x, self.position.y)
            .w_h(16.0, 16.0)
            .rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    let window = app.new_window().with_dimensions(200, 200).build().unwrap();
    let ball = Ball::new();
    Model { window, ball }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            match event {
                // KEY EVENTS
                KeyPressed(_key) => {}

                // MOUSE EVENTS
                MouseReleased(_button) => {}

                _other => (),
            }
        }
        // update gets called just before view every frame
        Event::Update(_dt) => {
            m.ball.update(app.window.rect());
        }
        _ => (),
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("noc_1_2 bouncingball vectors object");

    // Begin drawing
    let draw = app.draw();
    draw.rect().wh(app.window.size()).rgba(1.0, 1.0, 1.0, 0.03);

    m.ball.display(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
