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
    ball_r: f32,         // Ball radius
    damping: f32,        // Arbitary damping amount
    dragging: bool,
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
            ball_r: 48.0,   // Arbitrary ball radius
            damping: 0.995, // Arbitrary damping
            dragging: false,
        }
    }

    fn update(&mut self, rect: Rect) {
        // As long as we aren't dragging the pendulum, let it swing!
        if !self.dragging {
            let gravity = 0.4; // Arbitrary constant
            self.a_acceleration = (-1.0 * gravity / self.r) * self.angle.sin(); // Calculate acceleration (see: http://www.myphysicslab.com/pendulum1.html)
            self.a_velocity += self.a_acceleration; // Increment velocity
            self.a_velocity *= self.damping; // Arbitrary damping
            self.angle += self.a_velocity; // Increment angle
        }

        // Polar to cartesian conversion
        self.position = pt2(
            self.r * self.angle.sin(),
            rect.top() - self.r * self.angle.cos(),
        );
    }

    fn display(&self, draw: &DrawHolder) {
        // Draw the arm
        draw.line()
            .start(self.origin)
            .end(self.position)
            .color(BLACK)
            .stroke_weight(2.0);

        let c = if self.dragging { BLACK } else { GREY };

        // Draw the ball
        draw.ellipse()
            .xy(self.position)
            .w_h(self.ball_r, self.ball_r)
            .stroke(BLACK)
            .stroke_weight(2.0)
            .color(c)
            .stroke(BLACK);
    }

    // The methods below are for mouse interaction

    // This checks to see if we clicked on the pendulum ball
    fn clicked(&mut self, mx: f32, my: f32) {
        let d = pt2(mx, my).distance(pt2(self.position.x, self.position.y));
        if d < self.ball_r {
            self.dragging = true;
        }
    }

    // This tells us we are not longer clicking on the ball
    fn stop_dragging(&mut self) {
        if self.dragging {
            self.a_velocity = 0.0; // No velocity once you let go
            self.dragging = false;
        }
    }

    fn drag(&mut self, mx: f32, my: f32) {
        // If we are draging the ball, we calculate the angle between the
        // pendulum origin and mouse position
        // we assign that angle to the pendulum
        if self.dragging {
            let diff = self.origin - pt2(-mx, my);
            self.angle = (-1.0 * diff.y).atan2(diff.x) + deg_to_rad(90.0);
        }
    }
}

struct Model {
    pendulum: Pendulum,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(640, 360)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build();
    let pendulum = Pendulum::new(pt2(0.0, app.window_rect().top()), 175.0);

    Model { pendulum }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.pendulum.update(app.window_rect());
    m.pendulum.drag(app.mouse.x, app.mouse.y);
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.pendulum.display(&draw);



}

fn mouse_pressed(app: &App, m: &mut Model, _button: MouseButton) {
    m.pendulum.clicked(app.mouse.x, app.mouse.y);
}

fn mouse_released(_app: &App, m: &mut Model, _button: MouseButton) {
    m.pendulum.stop_dragging();
}
