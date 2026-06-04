// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 2-5: Forces (Gravity and Fluid Resistence) with Vectors
//
// Demonstration of multiple forces acting on bodies (Mover type)
// Bodies experience gravity continuously
// Bodies experience fluid resistance when in *water*
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    movers: Vec<Mover>,
    liquid: Liquid,
}

struct Mover {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    mass: f32,
}

// Liquid type
struct Liquid {
    // Liquid is a rectangle
    rect: geom::Rect,
    // Coefficient of drag
    c: f32,
}

impl Liquid {
    fn new(rect: geom::Rect, c: f32) -> Self {
        let rect = rect;
        let c = c;
        Liquid { rect, c }
    }

    // Is the Mover in the liquid?
    fn contains(&self, m: &Mover) -> bool {
        self.rect.contains(m.position)
    }

    // Calculate drag force
    fn drag(&self, m: &Mover) -> Vec2 {
        // Magnitude is coefficient * speed squared
        let speed = m.velocity.length();
        let drag_magnitude = self.c * speed * speed;

        // Direction is inverse of velocity
        let mut drag_force = m.velocity;
        drag_force *= -1.0;

        // Scale according to magnitude
        drag_force = drag_force.normalize();
        drag_force *= drag_magnitude;
        drag_force
    }

    fn display(&self, draw: &Draw) {
        draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).gray(0.1);
    }
}

impl Mover {
    fn new(m: f32, x: f32, y: f32) -> Self {
        // Mass is tied to size
        let mass = m;
        let position = pt2(x, y);
        let velocity = vec2(0.0, 0.0);
        let acceleration = vec2(0.0, 0.0);
        Mover {
            position,
            velocity,
            acceleration,
            mass,
        }
    }

    fn new_random(rect: &geom::Rect) -> Self {
        Mover::new(
            random_range(0.5f32, 4.0),
            random_range(rect.left(), rect.right()),
            rect.top(),
        )
    }

    // Newton's 2nd law: F = M * A
    // or A = F / M
    fn apply_force(&mut self, force: Vec2) {
        // Divide by mass
        let f = force / self.mass;
        // Accumulate all forces in acceleration
        self.acceleration += f;
    }

    fn update(&mut self) {
        // Velocity changes according to acceleration
        self.velocity += self.acceleration;
        // Position changes by velocity
        self.position += self.velocity;
        // We must clear acceleration each frame
        self.acceleration *= 0.0;
    }

    // Draw Mover
    fn display(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.position)
            .w_h(self.mass * 16.0, self.mass * 16.0)
            .srgba(0.0, 0.0, 0.0, 0.5)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }

    // Bounce off bottom of window
    fn check_edges(&mut self, rect: geom::Rect) {
        if self.position.y < rect.bottom() {
            self.velocity.y *= -0.9; // A little dampening when hitting the bottom
            self.position.y = rect.bottom();
        }
    }
}

fn model(app: &App) -> Model {
    let rect = geom::Rect::from_w_h(640.0, 360.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .mouse_pressed(mouse_pressed)
        .view(view)
        .build();

    // Nine moving bodies
    let movers = (0..9)
        .map(|_| Mover::new_random(&app.window_rect()))
        .collect();

    // Create an instance of our Liquid type
    let rect = geom::Rect::from_w_h(rect.w(), rect.h() * 0.5).align_bottom_of(rect);
    let liquid = Liquid::new(rect, 0.1);

    Model { movers, liquid }
}

fn mouse_pressed(app: &App, m: &mut Model, _button: MouseButton) {
    // Restart all the Mover objects randomly
    for mover in &mut m.movers {
        *mover = Mover::new_random(&app.window_rect());
    }
}

fn update(app: &App, m: &mut Model) {
    for i in 0..m.movers.len() {
        // Is the Mover in the liquid?
        if m.liquid.contains(&m.movers[i]) {
            let drag_force = m.liquid.drag(&m.movers[i]);
            // Apply drag force to Mover
            m.movers[i].apply_force(drag_force);
        }

        // Gravity is scaled by mass here!
        let gravity = vec2(0.0, -0.1 * m.movers[i].mass);

        // Apply gravity
        m.movers[i].apply_force(gravity);
        m.movers[i].update();
        m.movers[i].check_edges(app.window_rect());
    }
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // Draw water
    m.liquid.display(&draw);

    // Draw movers
    for mover in &m.movers {
        mover.display(&draw);
    }
}
