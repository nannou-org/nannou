// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-10: Exercise Particle Repel
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A simple particle type
#[derive(Clone)]
struct Particle {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
    life_span: f32,
    r: f32,
    idx: u64,
}

impl Particle {
    fn new(l: Point2, idx: u64) -> Self {
        let acceleration = vec2(0.0, 0.0);
        let velocity = vec2(random_f32() * 2.0 - 1.0, random_f32() * 2.0 - 1.0);
        let position = l;
        let life_span = 255.0;
        Particle {
            acceleration,
            velocity,
            position,
            life_span,
            r: 6.0,
            idx,
        }
    }

    fn intersects(&mut self, particles: &Vec<Particle>) {
        for i in 0..particles.len() {
            if particles[i].idx != self.idx {
                let mut dir = self.position - particles[i].position;
                if dir.magnitude() < self.r {
                    dir = dir.with_magnitude(0.5);
                    self.apply_force(dir);
                }
            }
        }
    }

    fn apply_force(&mut self, f: Vector2) {
        self.acceleration += f;
    }

    // Method to update position
    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position -= self.velocity;
        self.acceleration *= 0.0;
        self.life_span -= 0.5;
    }

    // Method to display
    fn display(&self, draw: &app::Draw) {
        draw.ellipse()
            .xy(self.position)
            .radius(self.r)
            .rgba(0.5, 0.5, 0.5, self.life_span / 255.0)
            .stroke(rgba(0.0, 0.0, 0.0, self.life_span / 255.0))
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

    fn add_particle(&mut self, idx: u64) {
        self.particles.push(Particle::new(self.origin, idx));
    }

    fn _apply_force(&mut self, f: Vector2) {
        for i in 0..self.particles.len() {
            self.particles[i].apply_force(f);
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

    fn intersection(&mut self) {
        let particles = self.particles.clone();
        for i in 0..self.particles.len() {
            self.particles[i].intersects(&particles);
        }
    }

    fn draw(&self, draw: &app::Draw) {
        for p in self.particles.iter() {
            p.display(&draw);
        }
    }
}

struct Model {
    ps: ParticleSystem,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();
    let ps = ParticleSystem::new(pt2(0.0, 0.0));
    Model { ps }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    let win = app.window_rect();
    m.ps.origin = pt2(
        random_range(win.left(), win.right()),
        random_range(win.bottom(), win.top()),
    );
    //let gravity = pt2(0.0, 0.1);
    //m.ps.apply_force(gravity);

    m.ps.add_particle(app.elapsed_frames());
    m.ps.update();
    m.ps.intersection();
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.ps.draw(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
