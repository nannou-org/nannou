// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-10: Motion 101 Acceleration
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    mover: Mover,
}

struct Mover {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    top_speed: f32,
}

impl Mover {
    fn new(rect: Rect<f32>) -> Self {
        let position = pt2(rect.x(), rect.y());
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
        self.velocity - limit_magnitude(self.velocity, self.top_speed);
        // Position changes velocity
        self.position += self.velocity;
    }

    fn display(&self, draw: &app::Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    let _window = app.new_window().with_dimensions(640, 360).build().unwrap();
    let mover = Mover::new(app.window_rect());
    Model { mover }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        m.mover.update(pt2(app.mouse.x, app.mouse.y));
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.mover.display(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}

// This will be a removed in favour of limir_magnitude method of vector
fn limit_magnitude(v: Vector2<f32>, limit: f32) -> Vector2<f32> {
    if v.magnitude() > limit {
        v.normalize() * limit
    } else {
        v
    }
}
