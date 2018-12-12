// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-02: Vector Particle
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    particles: Vec<Particle>,
}

// A simple particle type
struct Particle {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
    life_span: f32,
}

impl Particle {
    fn new(l: Point2) -> Self {
        let acceleration = vec2(0.0, 0.05);
        let velocity = vec2(random_f32() * 2.0 - 1.0, random_f32() - 1.0);
        let position = l;
        let life_span = 255.0;
        Particle {
            acceleration,
            velocity,
            position,
            life_span,
        }
    }

    // Method to update position
    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position -= self.velocity;
        self.life_span -= 2.0;
    }

    // Method to display
    fn display(&self, draw: &app::Draw) {
        let size = 5.0 + (255.0 - self.life_span) * 0.13;
        draw.ellipse().xy(self.position).w_h(size, size).rgba(
            (255.0 - self.life_span) / 255.0,
            self.velocity.x,
            0.5,
            self.life_span / 255.0,
        );
    }

    // Is the poarticel still useful?
    fn is_dead(&self) -> bool {
        if self.life_span < 0.0 {
            true
        } else {
            false
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window().with_dimensions(640, 360).view(view).build().unwrap();
    let particles = Vec::new();
    Model { particles }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.particles
        .push(Particle::new(pt2(0.0, app.window_rect().top() - 50.0)));
    for i in (0..m.particles.len()).rev() {
        m.particles[i].update();
        if m.particles[i].is_dead() {
            m.particles.remove(i);
        }
    }
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(BLACK);

    for p in m.particles.iter() {
        p.display(&draw);
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
