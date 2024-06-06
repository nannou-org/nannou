// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Pendulum

// A simple pendulum simulation
// Given a pendulum with an angle theta (0 being the pendulum at rest) and a radius r
// we can use sine to calculate the angular component of the gravitational force.

// Gravity Force = Mass * Gravitational Constant;
// Pendulum Force = Gravity Force * sine(theta)
// Angular Acceleration = Pendulum Force / Mass = Gravitational Constant * sine(theta);

// Note this is an ideal world scenario with no tension in the
// pendulum arm, a more realistic formula might be:
// Angular Acceleration = (G / R) * sine(theta)

// For a more substantial explanation, visit:
// http://www.myphysicslab.com/pendulum1.html

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A Simple Pendulum Module
// Includes functionality for user can click and drag the pendulum

struct Pendulum {
    position: Point2,    // position of pendulum ball
    origin: Point2,      // position of arm origin
    r: f32,              // Length of arm
    angle: f32,          // Pendulum arm angle
    a_velocity: f32,     // Angle velocity
    a_acceleration: f32, // Angle acceleration
    damping: f32,        // Arbitary damping amount
}

impl Pendulum {
    fn new(origin: Point2, r: f32) -> Self {
        Pendulum {
            position: pt2(0.0, 0.0),
            origin,
            r,
            angle: PI / 4.0,
            a_velocity: 0.0,
            a_acceleration: 0.0,
            damping: 0.995, // Arbitrary damping
        }
    }

    fn update(&mut self, rect: Rect) {
        let gravity = 0.4; // Arbitrary constant
        self.a_acceleration = (-1.0 * gravity / self.r) * self.angle.sin(); // Calculate acceleration (see: http://www.myphysicslab.com/pendulum1.html)
        self.a_velocity += self.a_acceleration; // Increment velocity
        self.a_velocity *= self.damping; // Arbitrary damping
        self.angle += self.a_velocity; // Increment angle

        // Polar to cartesian conversion
        self.position = pt2(
            self.r * self.angle.sin(),
            rect.top() - self.r * self.angle.cos(),
        );
    }

    fn display(&self, draw: &Draw) {
        // Draw the arm
        draw.line()
            .start(self.origin)
            .end(self.position)
            .color(BLACK)
            .stroke_weight(2.0);

        // Draw the ball
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .stroke(BLACK)
            .stroke_weight(2.0)
            .color(GREY)
            .stroke(BLACK);
    }
}

struct Model {
    pendulum: Pendulum,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();
    let pendulum = Pendulum::new(pt2(0.0, app.window_rect().top()), 175.0);

    Model { pendulum }
}

fn update(app: &App, m: &mut Model) {
    m.pendulum.update(app.window_rect());
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.pendulum.display(&draw);



}
