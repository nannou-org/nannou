// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Stay Within Walls
// "Made-up" Steering behavior to stay within walls

// Implements Craig Reynold's autonomous steering behaviors
// One vehicle "seeks"
// See: http://www.red3d.com/cwr/
extern crate nannou;

use nannou::prelude::*;
use nannou::Draw;

fn main() {
    nannou::app(model, event, view).run();
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
        let velocity = vec2(3.0, -2.0) * 5.0;
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
        // Update velocity and limit speed
        self.velocity = (self.velocity + self.acceleration).limit_magnitude(self.max_speed);
        self.position += self.velocity;
        // Reset accelerationelertion to 0 each cycle
        self.acceleration *= 0.0;
    }

    fn apply_force(&mut self, force: Vector2) {
        // We could add mass here if we want A = F / M
        self.acceleration += force;
    }
}

fn model(app: &App) -> Model {
    let _window = app.new_window().with_dimensions(640, 360).build().unwrap();
    let middle = app.window_rect().xy();
    let vehicle = Vehicle::new(middle.x, middle.y);
    let debug = true;
    let d = 25.0;
    Model { vehicle, debug, d }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    {
        // update gets called just before view every frame
        match event { 
            Event::Update(_update) => {
                boundaries(&mut m, &app.window_rect());
                m.vehicle.update();
            },
            Event::WindowEvent{ simple:Some(MousePressed(_button)), ..} => m.debug = !m.debug,
            _ => (),
        }
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let window = app.window_rect();

    if m.debug {
        draw.rect()
            .x_y(window.x(), window.y())
            .w(window.w() - m.d * 2.0)
            .h(window.h() - m.d * 2.0)
            .color(BLACK);
    }

    display(&m.vehicle, &draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
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
    let theta = (velocity.angle() + 90.0.to_radians()) * -1.0;
    let points = vec![
        pt3(0.0, -r * 2.0, 0.0),
        pt3(-r, r * 2.0, 0.0),
        pt3(*r, r * 2.0, 0.0),
    ];
    draw.polygon()
        .points(points)
        .xy(*position)
        .rgb(0.5, 0.5, 0.5)
        //TODO add outline
        //stroke(0),
        //strokeWeight(1),
        .rotate(theta);
}

fn boundaries(m: &mut Model, window_rect: &Rect) {
    let Model {
        d, ref mut vehicle, ..
    } = *m;

    let Vehicle {
        position,
        velocity,
        max_speed,
        max_force,
        ..
    } = *vehicle;

    let left = window_rect.left() + d;
    let right = window_rect.right() - d;
    let top = window_rect.top() - d;
    let bottom = window_rect.bottom() + d;

    let desired = match position {
        Vector2{x, ..} if x < left => Some(vec2(max_speed, velocity.y)),
        Vector2{x, ..} if x > right => Some(vec2(-max_speed, velocity.y)),
        Vector2{y, ..} if y < bottom => Some(vec2(velocity.x, max_speed)),
        Vector2{y, ..} if y > top => Some(vec2(velocity.x, -max_speed)),
        _ => None,
    };

    if let Some(desired) = desired {
        let mut desired = desired.normalize() * max_speed;
        let steer = (desired - velocity).limit_magnitude(max_force);
        vehicle.apply_force(steer);
    }
}
