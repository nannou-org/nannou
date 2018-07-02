// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-11: Motion 101 Acceleration Array
//
// Demonstration of the basics of motion with vector.
// A "Mover" object stores position, velocity, and acceleration as vectors
// The motion is controlled by affecting the acceleration
// (in this case towards the mouse)
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    movers: Vec<Mover>,
}

struct Mover {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    top_speed: f32,
}

impl Mover {
    fn new(rect: Rect<f32>) -> Self {
        let rand_x = random_range(rect.left(), rect.right());
        let rand_y = random_range(rect.top(), rect.bottom());
        let position = pt2(rand_x, rand_y);
        let velocity = vec2(0.0, 0.0);
        let acceleration = vec2(0.0, 0.0);
        let top_speed = 5.0;
        Mover {
            position,
            velocity,
            acceleration,
            top_speed,
        }
    }

    fn update(&mut self, mouse: Point2<f32>) {
        // Computer a vector that points from position to mouse
        self.acceleration = mouse - self.position;
        // Set magnitude of acceleration
        self.acceleration = self.acceleration.normalize() * 0.2;
        // Velocity chages according to acceleration
        self.velocity += self.acceleration;
        // Limit the velocity by top_speed
        self.velocity = vec2(
            self.velocity.x.min(self.top_speed),
            self.velocity.y.min(self.top_speed),
        );
        // Position changes velocity
        self.position += self.velocity;
    }

    fn display(&self, draw: &app::Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .rgba(0.5, 0.5, 0.5, 0.7);
    }
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(640.0, 360.0);
    let _window = app
        .new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .build()
        .unwrap();

    let movers = (0..20).map(|_| Mover::new(rect)).collect();
    Model { movers }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        for mover in &mut m.movers {
            mover.update(pt2(app.mouse.x, app.mouse.y));
        }
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    for mover in &m.movers {
        mover.display(&draw);
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
