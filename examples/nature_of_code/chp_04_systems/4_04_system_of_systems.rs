// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-04: System of Systems
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    systems: Vec<ParticleSystem>,
}

// A simple particle type
struct Particle {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    life_span: f32,
}

impl Particle {
    fn new(l: Point2<f32>) -> Self {
        let acceleration = vec2(0.0, 0.05);
        let velocity = vec2(random_f32() * 2.0 - 1.0, random_f32() - 2.0);
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
        let size = 12.0;
        draw.ellipse()
            .xy(self.position)
            .w_h(size, size)
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
    origin: Point2<f32>,
}

impl ParticleSystem {
    fn new(num: i32, position: Point2<f32>) -> Self {
        let origin = position; // An origin point for where particles are birthed
        let mut particles = Vec::new(); // Initialise the Vector
        for _i in 0..num {
            particles.push(Particle::new(origin)); // Add "num" amount of particles to the vector
        }
        ParticleSystem { origin, particles }
    }

    fn add_particle(&mut self) {
        self.particles.push(Particle::new(self.origin));
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

    // A method to test if the particle system still has particles
    fn _dead(&self) -> bool {
        if self.particles.is_empty() {
            true
        } else {
            false
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(640, 360)
        .mouse_pressed(mouse_pressed)
        .view(view)
        .build()
        .unwrap();
    let systems = Vec::new();
    Model { systems }
}

fn mouse_pressed(app: &App, m: &mut Model, _button: MouseButton) {
    m.systems
        .push(ParticleSystem::new(1, pt2(app.mouse.x, app.mouse.y)));
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    for ps in m.systems.iter_mut() {
        ps.add_particle();
        ps.update();
    }
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    for i in 0..m.systems.len() {
        m.systems[i].draw(&draw);
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
