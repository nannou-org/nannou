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
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    movers: Vec<Mover>,
}

struct Mover {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
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

    fn update(&mut self, mouse: Point2) {
        // Computer a vector that points from position to mouse
        self.acceleration = mouse - self.position;
        // Set magnitude of acceleration
        self.acceleration = self.acceleration.normalize_or_zero() * 0.2;
        // Velocity chages according to acceleration
        self.velocity += self.acceleration;
        // Limit the velocity by top_speed
        self.velocity = self.velocity.clamp_length_max(self.top_speed);
        // Position changes velocity
        self.position += self.velocity;
    }

    fn display(&self, draw: &DrawHolder) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .rgba(0.5, 0.5, 0.5, 0.7)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(640.0, 360.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .view(view)
        .build();

    let movers = (0..20).map(|_| Mover::new(rect)).collect();
    Model { movers }
}

// update gets called just before view every frame
fn update(app: &App, m: &mut Model, _update: Update) {
    for mover in &mut m.movers {
        mover.update(pt2(app.mouse.x, app.mouse.y));
    }
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    for mover in &m.movers {
        mover.display(&draw);
    }



}
