// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-01: Single Particle
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    p: Particle,
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
        draw.ellipse().xy(self.position).w_h(12.0, 12.0).rgba(
            0.5,
            0.5,
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
    let p = Particle::new(pt2(0.0, app.window_rect().top() - 20.0));
    Model { p }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.p.update();
    if m.p.is_dead() {
        m.p = Particle::new(pt2(0.0, app.window_rect().top() - 20.0));
    }
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.p.display(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
