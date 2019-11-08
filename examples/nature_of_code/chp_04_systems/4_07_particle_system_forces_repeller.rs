// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-07: Particle System Forces Repeller
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A very basic Repeller type
struct Repeller {
    // Gravitational Constant
    g: u64,
    // position
    position: Point2,
}

impl Repeller {
    fn new(x: f32, y: f32) -> Self {
        Repeller {
            g: 100,
            position: pt2(x, y),
        }
    }

    fn display(&self, draw: &app::Draw) {
        draw.ellipse()
            .xy(self.position)
            .radius(24.0)
            .color(GRAY)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }

    // Calculate a force to push particle away from repeller
    fn repel(&self, p: &Particle) -> Vector2 {
        let mut dir = self.position - p.position; // Calculate direction of force
        let mut d = dir.magnitude(); // Distance between objects
        dir = dir.normalize(); // Normalize vector (distance doesn't matter here, we just want this vector for direction)
        d = clamp(d, 5.0, 100.0); // Keep distance within a reasonable range
        let force = self.g as f32 / (d * d); // Repelling force is inversely proportional to distance
        dir *= vec2(force, force); // Get force vector --> magnitude * direction
        dir
    }
}

// A simple particle type
struct Particle {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
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

    fn apply_force(&mut self, f: Vector2) {
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
    fn display(&self, draw: &app::Draw) {
        draw.ellipse()
            .xy(self.position)
            .radius(6.0)
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

    fn add_particle(&mut self) {
        self.particles.push(Particle::new(self.origin));
    }

    // A function to apply a force to all Particles
    fn apply_force(&mut self, f: Vector2) {
        for p in self.particles.iter_mut() {
            p.apply_force(f);
        }
    }

    fn apply_repeller(&mut self, r: &Repeller) {
        for p in self.particles.iter_mut() {
            let force = r.repel(p);
            p.apply_force(force);
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

    fn draw(&self, draw: &app::Draw) {
        for p in self.particles.iter() {
            p.display(&draw);
        }
    }
}

struct Model {
    ps: ParticleSystem,
    repeller: Repeller,
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(640, 360)
        .view(view)
        .build()
        .unwrap();
    let ps = ParticleSystem::new(pt2(0.0, app.window_rect().top() - 50.0));
    let repeller = Repeller::new(-20.0, 0.0);
    Model { ps, repeller }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    m.ps.add_particle();

    // Apply gravity force to all Particles
    let gravity = pt2(0.0, 0.1);
    m.ps.apply_force(gravity);

    m.ps.apply_repeller(&m.repeller);
    m.ps.update();
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.repeller.display(&draw);
    m.ps.draw(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
