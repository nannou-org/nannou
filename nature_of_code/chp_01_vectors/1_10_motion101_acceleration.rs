// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-10: Motion 101 Acceleration
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    mover: Mover,
}

struct Mover {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    top_speed: f32,
}

impl Mover {
    fn new(rect: geom::Rect<f32>) -> Self {
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

    fn display(&self, draw: &Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .gray(0.5)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size(640, 360).view(view).build();
    let mover = Mover::new(app.window_rect());
    Model { mover }
}

fn update(app: &App, m: &mut Model) {
    // update gets called just before view every frame
    m.mover.update(pt2(app.mouse().x, app.mouse().y));
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.mover.display(&draw);
}
