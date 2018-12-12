// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 2-3: Forces Many Real Gravity
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    movers: Vec<Mover>,
}

struct Mover {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
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

    fn apply_force(&mut self, force: Vector2) {
        let f = force / self.mass;
        self.acceleration += f;
    }

    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position += self.velocity;
        self.acceleration *= 0.0;
    }

    fn display(&self, draw: &app::Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(self.mass * 16.0, self.mass * 16.0)
            .rgba(0.0, 0.0, 0.0, 0.5);
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
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(640.0, 360.0);
    app.new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .view(view)
        .build()
        .unwrap();

    let movers = (0..20)
        .map(|_| Mover::new(random_range(1.0f32, 4.0), rect.left(), rect.top()))
        .collect();
    Model { movers }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    for i in 0..m.movers.len() {
        let wind = vec2(0.01, 0.0);
        let gravity = vec2(0.0, -0.1 * m.movers[i].mass);
        m.movers[i].apply_force(wind);
        m.movers[i].apply_force(gravity);
        m.movers[i].update();
        m.movers[i].check_edges(app.window_rect());
    }
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
