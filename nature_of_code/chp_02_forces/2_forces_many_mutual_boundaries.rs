// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 2-0 Many Mutual Boundaries
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Mover {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    mass: f32,
}

impl Mover {
    fn new(m: f32, x: f32, y: f32) -> Self {
        let mass = m;
        let position = pt2(x, y);
        let velocity = vec2(0.0, 0.0);
        let acceleration = vec2(0.0, 0.0);
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

    fn display(&self, draw: &DrawHolder) {
        draw.ellipse()
            .xy(self.position)
            .w_h(self.mass * 16.0, self.mass * 16.0)
            .rgba(0.6, 0.6, 0.6, 0.7);
    }

    fn attract(&self, m: &Mover) -> Vec2 {
        let mut force = self.position - m.position; // Calculate direction of force
        let mut distance = force.length(); // Distance between objects
        distance = distance.max(5.0).min(25.0); // Limiting the distance to eliminate "extreme" results for very cose or very far object
        force = force.normalize(); // Normalize vector (distance doesn't matter, we just want this vector for direction)
        let g = 0.4;
        let strength = (g * self.mass * m.mass) / (distance * distance); // Calculate gravitational force magnitude
        force * strength // Get force vector --> magnitude * direction
    }

    fn boundaries(&mut self, rect: Rect) {
        let d = 50.0;
        let mut force = vec2(0.0, 0.0);

        if self.position.x < rect.pad(d).left() {
            force.x = 1.0;
        } else if self.position.x > rect.pad(d).right() {
            force.x = -1.0;
        }
        if self.position.y > rect.pad(d).top() {
            force.y = -1.0;
        } else if self.position.y < rect.pad(d).bottom() {
            force.y = 1.0;
        }
        if force.length() > 0.0 {
            force = force.normalize();
            force *= 0.1;
            self.apply_force(force);
        }
    }
}

struct Model {
    movers: Vec<Mover>,
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(640.0, 360.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .view(view)
        .build();

    let movers = (0..20)
        .map(|_| {
            Mover::new(
                random_range(1.0f32, 2.0),
                random_range(rect.left(), rect.right()),
                random_range(rect.top(), rect.bottom()),
            )
        })
        .collect();

    Model { movers }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    for i in 0..m.movers.len() {
        for j in 0..m.movers.len() {
            if i != j {
                let force = m.movers[j].attract(&m.movers[i]);
                m.movers[i].apply_force(force);
            }
        }
        m.movers[i].boundaries(app.window_rect());
        m.movers[i].update();
    }
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // Draw movers
    for mover in &m.movers {
        mover.display(&draw);
    }



}
