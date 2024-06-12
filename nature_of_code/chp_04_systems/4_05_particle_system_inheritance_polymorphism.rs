// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-05: Particle System Inheritance PolyMorphism
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

enum ParticleType {
    Ellipse,
    Confetti,
}

// A simple particle type
struct Particle {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    life_span: f32,
    particle_type: ParticleType,
}

impl Particle {
    fn new(l: Point2, particle_type: ParticleType) -> Self {
        let acceleration = vec2(0.0, 0.05);
        let velocity = vec2(random_f32() * 2.0 - 1.0, random_f32() - 2.0);
        let position = l;
        let life_span = 255.0;
        Particle {
            acceleration,
            velocity,
            position,
            life_span,
            particle_type,
        }
    }

    // Method to update position
    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position -= self.velocity;
        self.life_span -= 2.0;
    }

    // Method to display
    fn display(&self, draw: &Draw, win: geom::Rect) {
        match &self.particle_type {
            ParticleType::Ellipse => {
                draw.ellipse()
                    .xy(self.position)
                    .radius(6.0)
                    .rgba(0.5, 0.5, 0.5, self.life_span / 255.0)
                    .stroke(Color::srgba(0.0, 0.0, 0.0, self.life_span / 255.0))
                    .stroke_weight(2.0);
            }
            ParticleType::Confetti => {
                let theta = map_range(self.position.x, win.left(), win.right(), 0.0, PI * 2.0);
                draw.rect()
                    .xy(self.position)
                    .w_h(12.0, 12.0)
                    .rgba(0.5, 0.5, 0.5, self.life_span / 255.0)
                    .stroke(Color::srgba(0.0, 0.0, 0.0, self.life_span / 255.0))
                    .stroke_weight(2.0)
                    .rotate(theta);
            }
        }
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

struct ParticleSystem {
    particles: Vec<Particle>,
    origin: Point2,
}

impl ParticleSystem {
    fn new(position: Point2) -> Self {
        let origin = position;
        let particles = Vec::new();
        ParticleSystem { origin, particles }
    }

    fn add_particle(&mut self) {
        let particle_type = if random_f32() < 0.5 {
            ParticleType::Ellipse
        } else {
            ParticleType::Confetti
        };
        self.particles
            .push(Particle::new(self.origin, particle_type));
    }

    fn update(&mut self) {
        for i in (0..self.particles.len()).rev() {
            self.particles[i].update();
            if self.particles[i].is_dead() {
                self.particles.remove(i);
            }
        }
    }

    fn draw(&self, draw: &Draw, win: Rect) {
        for i in (0..self.particles.len()).rev() {
            self.particles[i].display(&draw, win);
        }
    }
}

struct Model {
    ps: ParticleSystem,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();
    let (_w, h) = app.window_rect().w_h();
    let ps = ParticleSystem::new(pt2(0.0, (h as f32 / 2.0) - 50.0));
    Model { ps }
}

fn update(_app: &App, m: &mut Model) {
    m.ps.add_particle();
    m.ps.update();
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.ps.draw(&draw, app.window_rect());
}
