// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-8: Motion 101 Acceleration
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
    fn new(_rect: geom::Rect) -> Self {
        let position = pt2(0.0, 0.0);
        let velocity = vec2(0.0, 0.0);
        let acceleration = vec2(-0.001, 0.01);
        let top_speed = 10.0;
        Mover {
            position,
            velocity,
            acceleration,
            top_speed,
        }
    }

    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.velocity = vec2(
            self.velocity.x.min(self.top_speed),
            self.velocity.y.min(self.top_speed),
        );
        self.position += self.velocity;
    }

    fn check_edges(&mut self, rect: geom::Rect) {
        if self.position.x > rect.right() {
            self.position.x = rect.left();
        } else if self.position.x < rect.left() {
            self.position.x = rect.right();
        }
        if self.position.y > rect.top() {
            self.position.y = rect.bottom();
        } else if self.position.y < rect.bottom() {
            self.position.y = rect.top();
        }
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
    app.new_window().size(640, 360).view(view).build();
    let mover = Mover::new(app.window_rect());
    Model { mover }
}

fn update(app: &App, m: &mut Model) {
    m.mover.update();
    m.mover.check_edges(app.window_rect());
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.rect()
        .wh(app.window_rect().wh())
        .rgba(1.0, 1.0, 1.0, 0.03);

    m.mover.display(&draw);
}
