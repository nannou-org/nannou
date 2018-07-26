// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Seek_Arrive

// Implements Craig Reynold's autonomous steering behaviors
// One vehicle "seeks"
// See: http://www.red3d.com/cwr/
extern crate nannou;

use nannou::prelude::*;
use nannou::Draw;
use std::collections::VecDeque;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    vehicle: Vehicle,
}

struct Vehicle {
    history: VecDeque<Vector2>,
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
        let history = VecDeque::<Vector2>::with_capacity(100);
        let position = vec2(x, y);
        let velocity = vec2(0.0, -2.0);
        let acceleration = vec2(0.0, 0.0);
        let r = 6.0;
        let max_force = 0.1;
        let max_speed = 4.0;

        Vehicle {
            history,
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
        self.history.push_back(self.position);
        if self.history.len() > 100 {
            self.history.pop_front();
        }
    }

    fn apply_force(&mut self, force: Vector2) {
        // We could add mass here if we want A = F / M
        self.acceleration += force;
    }
}

fn model(app: &App) -> Model {
    let _window = app.new_window().with_dimensions(800, 360).build().unwrap();
    let middle = app.window_rect().xy();
    let vehicle = Vehicle::new(middle.x, middle.y);
    Model { vehicle }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    {
        let Model {
            ref mut vehicle, ..
        } = m;
        let mouse = vec2(app.mouse.x, app.mouse.y);
        // update gets called just before view every frame
        if let Event::Update(_update) = event {
            seek(vehicle, mouse);
            vehicle.update();
        }
    }
    m
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
fn seek(vehicle: &mut Vehicle, target: Vector2) {
    let steer = {
        let Vehicle {
            ref position,
            ref velocity,
            ref max_speed,
            ref max_force,
            ..
        } = vehicle;
        // A vector pointing from the position to the target
        // Normalize desired and scale to maximum speed
        let desired = (target - *position).normalize().map(|i| i * max_speed);

        // Steering = Desired minus velocity
        // Limit to maximum steering force
        (desired - *velocity).limit_magnitude(*max_force)
    };

    vehicle.apply_force(steer);
}

fn display(vehicle: &Vehicle, draw: &Draw) {
    let Vehicle {
        history,
        position,
        velocity,
        r,
        ..
    } = vehicle;

    let thickness = 1.0;
    if history.len() > 1 {
        let vertices = history.iter().map(|v| {
            geom::vertex::Rgba(pt2(v.x, v.y), Rgba::new(0.0, 0.0, 0.0, 1.0))
        }); 
    draw.polyline().vertices(thickness, vertices);
        //TODO change to outline
        //stroke(0),
        //strokeWeight(1),
    }

    // Draw a triangle rotated in the direction of velocity
    // This calculation is wrong
    let theta = (velocity.angle() + PI/2.0) * -1.0;
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
