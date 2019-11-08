// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-06: Exercise Shatter
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A simple particle type
struct Particle {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
    life_span: f32,
    r: f32,
}

impl Particle {
    fn new(x: f32, y: f32, r: f32) -> Self {
        let acceleration = vec2(0.0, 0.05);
        let velocity = vec2(random_f32() * 2.0 - 1.0, random_f32() - 1.0);
        let position = pt2(x, y);
        let life_span = 255.0;
        Particle {
            acceleration,
            velocity,
            position,
            life_span,
            r,
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
        draw.rect().xy(self.position).w_h(self.r, self.r).rgba(
            0.0,
            0.0,
            0.0,
            self.life_span / 255.0,
        );
    }

    // Is the particle still useful?
    fn _is_dead(&self) -> bool {
        if self.life_span < 0.0 {
            true
        } else {
            false
        }
    }
}

struct ParticleSystem {
    particles: Vec<Particle>,
    intact: bool,
}

impl ParticleSystem {
    fn new(x: f32, y: f32, r: f32) -> Self {
        let particles = Vec::new();
        let rows = 20;
        let cols = 20;

        let mut ps = ParticleSystem {
            particles,
            intact: true,
        };

        for i in 0..(rows * cols) {
            ps.add_particle(x + (i % cols) as f32 * r, y - (i / rows) as f32 * r, r);
        }

        ps
    }

    fn add_particle(&mut self, x: f32, y: f32, r: f32) {
        self.particles.push(Particle::new(x, y, r));
    }

    fn shatter(&mut self) {
        self.intact = false;
    }

    fn update(&mut self) {
        if !self.intact {
            for i in (0..self.particles.len()).rev() {
                self.particles[i].update();
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
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(640, 360)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build()
        .unwrap();
    let win = app.window_rect();
    let ps = ParticleSystem::new(win.left() + 100.0, win.top() - 100.0, 5.0);
    Model { ps }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    m.ps.update();
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.ps.draw(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, m: &mut Model, _button: MouseButton) {
    m.ps.shatter();
}
