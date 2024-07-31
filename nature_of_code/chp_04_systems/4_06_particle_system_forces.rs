// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-06: Particle System Forces
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A simple particle type
struct Particle {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    life_span: f32,
    mass: f32,
}

impl Particle {
    fn new(l: Point2) -> Self {
        let acceleration = vec2(0.0, 0.0);
        let velocity = vec2(random_f32() * 2.0 - 1.0, random_f32() * 2.0 - 2.0);
        let position = l;
        let life_span = 255.0;
        let mass = 1.0; // Let's do something better here!

        Particle {
            acceleration,
            velocity,
            position,
            life_span,
            mass,
        }
    }

    fn apply_force(&mut self, f: Vec2) {
        self.acceleration += f / self.mass;
    }

    // Method to update position
    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position -= self.velocity;
        self.acceleration *= 0.0;
        self.life_span -= 2.0;
    }

    // Method to display
    fn display(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.position)
            .radius(6.0)
            .srgba(0.5, 0.5, 0.5, self.life_span / 255.0)
            .stroke(Color::srgba(0.0, 0.0, 0.0, self.life_span / 255.0))
            .stroke_weight(2.0);
    }

    // Is the particle still useful?
    fn is_dead(&self) -> bool {
        self.life_span < 0.0
    }
}

struct ParticleSystem {
    particles: Vec<Particle>,
    pub origin: Point2,
}

impl ParticleSystem {
    fn new(position: Point2) -> Self {
        let origin = position;
        let particles = Vec::new();
        ParticleSystem { origin, particles }
    }

    fn add_particle(&mut self) {
        self.particles.push(Particle::new(self.origin));
    }

    // A function to apply a force to all Particles
    fn apply_force(&mut self, f: Vec2) {
        for p in self.particles.iter_mut() {
            p.apply_force(f);
        }
    }

    fn update(&mut self) {
        for i in (0..self.particles.len()).rev() {
            self.particles[i].update();
            if self.particles[i].is_dead() {
                self.particles.remove(i);
            }
        }
    }

    fn draw(&self, draw: &Draw) {
        for p in self.particles.iter() {
            p.display(draw);
        }
    }
}

struct Model {
    ps: ParticleSystem,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build();
    let ps = ParticleSystem::new(pt2(0.0, app.window_rect().top() - 50.0));
    Model { ps }
}

fn update(_app: &App, m: &mut Model) {
    let gravity = pt2(0.0, 0.1);
    m.ps.apply_force(gravity);

    m.ps.add_particle();
    m.ps.update();
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.ps.draw(&draw);
}
