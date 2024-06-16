// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-10: Exercise Particle Intersection
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A simple particle type
#[derive(Clone)]
struct Particle {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    life_span: f32,
    r: f32,
    highlight: bool,
    idx: u64,
}

impl Particle {
    fn new(l: Point2, idx: u64) -> Self {
        let acceleration = vec2(0.0, 0.05);
        let velocity = vec2(random_f32() * 2.0 - 1.0, random_f32() - 1.0);
        let position = l;
        let life_span = 255.0;
        Particle {
            acceleration,
            velocity,
            position,
            life_span,
            r: 6.0,
            highlight: false,
            idx,
        }
    }

    fn intersects(&mut self, particles: &Vec<Particle>) {
        self.highlight = false;
        for i in 0..particles.len() {
            if particles[i].idx != self.idx {
                let d = particles[i].position.distance(self.position);
                if d < self.r + particles[i].r {
                    self.highlight = true;
                }
            }
        }
    }

    fn _apply_force(&mut self, f: Vec2) {
        self.acceleration += f;
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
        let c = if self.highlight {
            rgba(0.5, 0.0, 0.0, 1.0)
        } else {
            rgba(0.5, 0.5, 0.5, self.life_span / 255.0)
        };

        draw.ellipse()
            .xy(self.position)
            .radius(self.r)
            .color(c)
            .stroke(Color::srgba(0.0, 0.0, 0.0, self.life_span / 255.0))
            .stroke_weight(2.0);
    }

    // Is the particle still useful?
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
    pub origin: Point2,
}

impl ParticleSystem {
    fn new(position: Point2) -> Self {
        let origin = position;
        let particles = Vec::new();
        ParticleSystem { origin, particles }
    }

    fn add_particle(&mut self, frame_num: u64) {
        self.particles.push(Particle::new(self.origin, frame_num));
    }

    fn update(&mut self) {
        for i in (0..self.particles.len()).rev() {
            self.particles[i].update();
            if self.particles[i].is_dead() {
                self.particles.remove(i);
            }
        }
    }

    fn intersection(&mut self) {
        let particles = self.particles.clone();
        for i in 0..self.particles.len() {
            self.particles[i].intersects(&particles);
        }
    }

    fn draw(&self, draw: &Draw) {
        for p in self.particles.iter() {
            p.display(&draw);
        }
    }
}

struct Model {
    ps: ParticleSystem,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build();
    let ps = ParticleSystem::new(pt2(0.0, 0.0));
    Model { ps }
}

fn update(app: &App, m: &mut Model) {
    m.ps.origin = pt2(app.mouse().x, app.mouse().y);
    m.ps.add_particle(app.elapsed_frames());
    m.ps.update();
    m.ps.intersection();
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.ps.draw(&draw);
}
