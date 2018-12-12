// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Seeking "vehicle" follows the mouse position

// Implements Craig Reynold's autonomous steering behaviors
// One vehicle "seeks"
// See: http://www.red3d.com/cwr/
extern crate nannou;

use nannou::prelude::*;
use nannou::Draw;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    vehicle: Vehicle,
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
        let velocity = vec2(0.0, -2.0);
        let acceleration = vec2(0.0, 0.0);
        let r = 6.0;
        let max_force = 0.1;
        let max_speed = 4.0;

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
}

fn model(app: &App) -> Model {
    app.new_window().with_dimensions(640, 360).view(view).build().unwrap();
    let middle = app.window_rect().xy();
    let vehicle = Vehicle::new(middle.x, middle.y);
    Model { vehicle }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    seek(&mut m.vehicle, app.mouse.position());
    m.vehicle.update();
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let mouse = vec2(app.mouse.x, app.mouse.y);

    draw.ellipse()
        // Missing Stroke
        .x_y(mouse.x, mouse.y)
        .radius(48.0)
        .rgb(0.78, 0.78, 0.78);
    display(&m.vehicle, &draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}

// A method that calculates a steering force towards a target
// STEER = DESIRED MINUS VELOCITY
fn seek(vehicle: &mut Vehicle, target: Point2) {
    let steer = {
        let Vehicle {
            ref position,
            ref velocity,
            ref max_speed,
            ref max_force,
            ..
        } = vehicle;
        // A vector pointing from the position to the target
        // Scale to maximum speed
        let desired = (target - *position).with_magnitude(*max_speed);

        // Steering = Desired minus velocity
        // Limit to maximum steering force
        (desired - *velocity).limit_magnitude(*max_force)
    };

    vehicle.apply_force(steer);
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
