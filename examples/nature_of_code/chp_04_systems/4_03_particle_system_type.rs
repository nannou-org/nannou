// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-03: Particle System Type
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    ps: ParticleSystem,
}

// A simple particle type
struct Particle {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    life_span: f32,
}

impl Particle {
    fn new(l: Vector2<f32>) -> Self {
        let acceleration = Vector2::new(0.0, 0.05);
        let velocity = Vector2::new(random::<f32>() * 2.0 - 1.0, random::<f32>() - 1.0);
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
            .x_y(self.position.x, self.position.y)
            .w_h(size, size)
            .rgba(0.5, 0.5, 0.5, self.life_span / 255.0);
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
    origin: Vector2<f32>,
}

impl ParticleSystem {
    fn new(position: Vector2<f32>) -> Self {
        let origin = position;
        let particles = Vec::new();
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
}

fn model(app: &App) -> Model {
    let _window = app.new_window().with_dimensions(640, 360).build().unwrap();
    let (_w, h) = app.window_rect().w_h();
    let ps = ParticleSystem::new(Vector2::new(0.0, (h as f32 / 2.0) - 50.0));
    Model { ps }
}

fn event(_app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        m.ps.add_particle();
        m.ps.update();
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(0.0, 0.0, 0.0);

    m.ps.draw(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
