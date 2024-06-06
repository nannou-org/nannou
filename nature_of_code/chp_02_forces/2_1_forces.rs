// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 2-1: Forces
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
    mass: f32,
}

impl Mover {
    fn new(rect: Rect) -> Self {
        let position = pt2(rect.left() + 30.0, rect.top() - 30.0);
        let velocity = vec2(0.0, 0.0);
        let acceleration = vec2(0.0, 0.0);
        let mass = 1.0;
        Mover {
            position,
            velocity,
            acceleration,
            mass,
        }
    }

    fn apply_force(&mut self, force: Vec2) {
        let f = force / self.mass;
        self.acceleration += f;
    }

    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position += self.velocity;
        self.acceleration *= 0.0;
    }

    fn check_edges(&mut self, rect: Rect) {
        if self.position.x > rect.right() {
            self.position.x = rect.right();
            self.velocity.x *= -1.0;
        } else if self.position.x < rect.left() {
            self.velocity.x *= -1.0;
            self.position.x = rect.left();
        }
        if self.position.y < rect.bottom() {
            self.velocity.y *= -1.0;
            self.position.y = rect.bottom();
        }
    }

    fn display(&self, draw: &Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .gray(0.3)
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
    let mover = Mover::new(rect);
    Model { mover }
}

fn update(app: &App, m: &mut Model) {
    let wind = vec2(0.01, 0.0);
    let gravity = vec2(0.0, -0.1);
    m.mover.apply_force(wind);
    m.mover.apply_force(gravity);
    m.mover.update();
    m.mover.check_edges(app.window_rect());
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.mover.display(&draw);



}
