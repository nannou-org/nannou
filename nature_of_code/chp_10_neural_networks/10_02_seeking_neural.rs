// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// A Vehicle controlled by a Perceptron
//
// example 10-02: Seeking Neural
use nannou::prelude::*;
use nannou::Draw;

fn main() {
    nannou::app(model).update(update).run();
}

struct Perceptron {
    weights: Vec<f32>, // Array of weights for inputs
    c: f32,            // learning constant
}

impl Perceptron {
    // Perceptron is created with n weights and learning constant
    fn new(n: u32, c: f32) -> Self {
        // Start with random weights
        let weights = (0..n).map(|_| random_f32()).collect();

        Perceptron { weights, c }
    }

    // Function to train the Perceptron
    // Weights are adjusted based on "desired" answer
    fn train(&mut self, forces: &Vec<Vec2>, error: Vec2) {
        for i in 0..self.weights.len() {
            self.weights[i] += self.c * error.x * forces[i].x;
            self.weights[i] += self.c * error.y * forces[i].y;
            self.weights[i] = clamp(self.weights[i], 0.0, 1.0);
        }
    }

    // Give me a steering result
    fn feedforward(&self, forces: &Vec<Vec2>) -> Vec2 {
        // Sum all the values
        let mut sum = vec2(0.0, 0.0);
        for i in 0..self.weights.len() {
            sum += forces[i] * self.weights[i];
        }
        sum
    }
}

struct Vehicle {
    brain: Perceptron, // Vehicle now has a brain!
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    r: f32,
    max_force: f32, // Maximum steering force
    max_speed: f32, // Maximum speed
}

impl Vehicle {
    fn new(n: u32, x: f32, y: f32) -> Self {
        Vehicle {
            brain: Perceptron::new(n, 0.001),
            position: vec2(x, y),
            velocity: vec2(0.0, 0.0),
            acceleration: vec2(0.0, 0.0),
            r: 3.0,
            max_force: 0.1,
            max_speed: 4.0,
        }
    }

    // Method to update position
    fn update(&mut self, win: Rect) {
        // Update velocity
        self.velocity += self.acceleration;
        // Limit speed
        self.velocity = self.velocity.normalize() * self.max_speed;
        self.position += self.velocity;
        // Reset accelerationelertion to 0 each cycle
        self.acceleration *= 0.0;

        self.position.x = clamp(self.position.x, win.left(), win.right());
        self.position.y = clamp(self.position.y, win.bottom(), win.top());
    }

    fn apply_force(&mut self, force: Vec2) {
        // We could add mass here if we want A = F / M
        self.acceleration += force;
    }

    // Here is where the brain processes everything
    fn steer(&mut self, targets: &Vec<Vec2>, desired: &Vec2) {
        // Make an vector of forces
        let mut forces = vec![vec2(0.0, 0.0); targets.len()];
        // Steer towards all targets
        for i in 0..forces.len() {
            forces[i] = self.seek(targets[i]);
        }

        // That array of forces is the input to the brain
        let result = self.brain.feedforward(&forces);

        // Use the result to steer the vehicle
        self.apply_force(result);

        // Train the brain according to the error
        let error = *desired - self.position;
        self.brain.train(&forces, error);
    }

    // A method that calculates a steering force towards a target
    // STEER = DESIRED MINUS VELOCITY
    fn seek(&mut self, target: Vec2) -> Vec2 {
        // A vector pointing from the position to the target
        let mut desired = target - self.position;
        // Normalize desired and scale to maximum speed
        desired = desired.normalize();
        desired *= self.max_speed;
        // Steering = Desired minus velocity
        // Limit to maximum steering force
        let steer = (desired - self.velocity).clamp_length_max(self.max_force);
        steer
    }

    fn display(&self, draw: &Draw) {
        // Draw a triangle rotated in the direction of velocity
        // This calculation is wrong
        let theta = (self.velocity.angle() + PI / 2.0) * -1.0;
        let points = vec![
            pt2(0.0, -self.r * 2.0),
            pt2(-self.r, self.r * 2.0),
            pt2(self.r, self.r * 2.0),
        ];
        draw.polygon()
            .stroke(BLACK)
            .points(points)
            .xy(self.position)
            .rgb(0.5, 0.5, 0.5)
            .rotate(theta);
    }
}

struct Model {
    v: Vehicle, // A list of points we will use to "train" the perceptron
    desired: Vec2,
    targets: Vec<Vec2>,
    num_targets: u32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(640, 360)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build();

    let num_targets = 8;

    // The Vehicle's desired position
    let desired = vec2(0.0, 0.0);

    let win = app.window_rect();

    // Create the Vehicle (it has to know about the number of targets
    // in order to configure its brain)
    let v = Vehicle::new(
        num_targets,
        random_range(win.left(), win.right()),
        random_range(win.top(), win.bottom()),
    );
    let mut model = Model {
        v,
        desired,
        targets: Vec::new(),
        num_targets,
    };

    // Create a list of targets
    make_targets(&mut model, &win);

    model
}

// Make a random ArrayList of targets to steer towards
fn make_targets(m: &mut Model, win: &Rect) {
    m.targets = (0..m.num_targets)
        .map(|_| {
            vec2(
                random_range(win.left(), win.right()),
                random_range(win.bottom(), win.top()),
            )
        })
        .collect();
}

fn update(app: &App, m: &mut Model) {
    // Update the Vehicle
    m.v.steer(&m.targets, &m.desired);
    m.v.update(app.window_rect());
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // Draw a circle to show the Vehicle's goal
    draw.ellipse()
        .xy(m.desired)
        .radius(18.0)
        .rgba(0.0, 0.0, 0.0, 0.4)
        .stroke(BLACK)
        .stroke_weight(2.0);

    // Draw the targets
    for target in &m.targets {
        draw.ellipse()
            .xy(*target)
            .radius(8.0)
            .stroke(BLACK)
            .stroke_weight(2.0)
            .no_fill();

        draw.line()
            .start(pt2(target.x, target.y - 16.0))
            .end(pt2(target.x, target.y + 16.0))
            .color(BLACK)
            .stroke_weight(2.0);

        draw.line()
            .start(pt2(target.x - 16.0, target.y))
            .end(pt2(target.x + 16.0, target.y))
            .color(BLACK)
            .stroke_weight(2.0);
    }
    m.v.display(&draw);
}

fn mouse_pressed(app: &App, m: &mut Model, _button: MouseButton) {
    make_targets(m, &app.window_rect());
}
