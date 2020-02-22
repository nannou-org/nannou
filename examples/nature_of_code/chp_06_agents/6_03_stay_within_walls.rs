// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Stay Within Walls
// "Made-up" Steering behavior to stay within walls

// Implements Craig Reynold's autonomous steering behaviors
// One vehicle "seeks"
// See: http://www.red3d.com/cwr/
use nannou::prelude::*;
use nannou::Draw;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    vehicle: Vehicle,
    debug: bool,
    d: f32,
}

struct Vehicle {
    position: Vector2,
    velocity: Vector2,
    acceleration: Vector2,
    r: f32,
    // Maximum steering force
    max_force: f32,
    // Maximum speed
    max_speed: f32,
}

impl Vehicle {
    fn new(x: f32, y: f32) -> Self {
        let position = vec2(x, y);
        let velocity = vec2(3.0, -2.0);
        let acceleration = vec2(0.0, 0.0);
        let r = 6.0;
        let max_force = 0.15;
        let max_speed = 3.0;

        Vehicle {
            position,
            velocity,
            acceleration,
            r,
            max_force,
            max_speed,
        }
    }

    // Method to update position
    fn update(&mut self) {
        // Update velocity
        self.velocity += self.acceleration;
        // Limit speed
        self.velocity.limit_magnitude(self.max_speed);
        self.position += self.velocity;
        // Reset accelerationelertion to 0 each cycle
        self.acceleration *= 0.0;
    }

    fn apply_force(&mut self, force: Vector2) {
        // We could add mass here if we want A = F / M
        self.acceleration += force;
    }

    fn boundaries(&mut self, d: f32, win: &Rect) {
        let left = win.left() + d;
        let right = win.right() - d;
        let top = win.top() - d;
        let bottom = win.bottom() + d;

        let desired = match self.position {
            Vector2 { x, .. } if x < left => Some(vec2(self.max_speed, self.velocity.y)),
            Vector2 { x, .. } if x > right => Some(vec2(-self.max_speed, self.velocity.y)),
            Vector2 { y, .. } if y < bottom => Some(vec2(self.velocity.x, self.max_speed)),
            Vector2 { y, .. } if y > top => Some(vec2(self.velocity.x, -self.max_speed)),
            _ => None,
        };

        if let Some(desired) = desired {
            let desired = desired.normalize() * self.max_speed;
            let steer = (desired - self.velocity).limit_magnitude(self.max_force);
            self.apply_force(steer);
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(640, 360)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build()
        .unwrap();
    let middle = app.window_rect().xy();
    let vehicle = Vehicle::new(middle.x, middle.y);
    let debug = false;
    let d = 25.0;
    Model { vehicle, debug, d }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.vehicle.boundaries(m.d, &app.window_rect());
    m.vehicle.update();
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let win = app.window_rect();
    if m.debug {
        draw.rect()
            .x_y(win.x(), win.y())
            .w(win.w() - m.d * 2.0)
            .h(win.h() - m.d * 2.0)
            .no_fill()
            .stroke(GREY);
    }

    display(&m.vehicle, &draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn display(vehicle: &Vehicle, draw: &Draw) {
    let Vehicle {
        position,
        velocity,
        r,
        ..
    } = vehicle;
    // Draw a triangle rotated in the direction of velocity
    // This calculation is wrong
    let theta = (velocity.angle() + PI / 2.0) * -1.0;
    let points = vec![pt2(0.0, -r * 2.0), pt2(-r, r * 2.0), pt2(*r, r * 2.0)];
    draw.polygon()
        .stroke(BLACK)
        .stroke_weight(1.0)
        .points(points)
        .xy(*position)
        .rgb(0.5, 0.5, 0.5)
        .rotate(theta);
}

fn mouse_pressed(_app: &App, m: &mut Model, _button: MouseButton) {
    m.debug = !m.debug;
}
