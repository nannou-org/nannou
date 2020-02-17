// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 4-01: Single Particle
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
        draw.ellipse()
            .xy(self.position)
            .w_h(12.0, 12.0)
            .rgba(0.5, 0.5, 0.5, self.life_span / 255.0)
            .stroke(rgba(0.0, 0.0, 0.0, self.life_span / 255.0))
            .stroke_weight(2.0);
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

struct Model {
    p: Particle,
    mouse_down: bool,
}

fn model(app: &App) -> Model {
    app.new_window()
        .dimensions(800, 200)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build()
        .unwrap();
    let p = Particle::new(pt2(0.0, app.window_rect().top() - 20.0));
    Model {
        p,
        mouse_down: false,
    }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    if m.mouse_down {
        m.p.update();
        if m.p.is_dead() {
            println!("Particle dead!");
        }
    }
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();

    if app.elapsed_frames() == 1 {
        draw.background().color(WHITE);
    }

    if m.mouse_down {
        draw.rect()
            .wh(app.window_rect().wh())
            .rgba(1.0, 1.0, 1.0, 0.03);
        m.p.display(&draw);
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, m: &mut Model, _button: MouseButton) {
    m.mouse_down = true;
}

fn mouse_released(_app: &App, m: &mut Model, _button: MouseButton) {
    m.mouse_down = false;
}
