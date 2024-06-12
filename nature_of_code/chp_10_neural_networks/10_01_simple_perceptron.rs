// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Simple Perceptron Example
// See: http://en.wikipedia.org/wiki/Perceptron
// Code based on text "Artificial Intelligence", George Luger
//
// example 10-01: Simple Perceptron
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A class to describe a training point
// Has an x and y, a "bias" (1) and known output
// Could also add a variable for "guess" but not required here

struct Trainer {
    inputs: Vec<f32>,
    answer: i32,
}

impl Trainer {
    fn new(x: f32, y: f32, a: i32) -> Self {
        let inputs = vec![x, y, 1.0];
        Trainer { inputs, answer: a }
    }
}

struct Perceptron {
    weights: Vec<f32>, // Array of weights for inputs
    c: f32,            // learning constant
}

impl Perceptron {
    // Perceptron is created with n weights and learning constant
    fn new(n: i32, c: f32) -> Self {
        // Start with random weights
        let weights = (0..n).map(|_| random_range(-1.0, 1.0)).collect();

        Perceptron { weights, c }
    }

    // Function to train the Perceptron
    // Weights are adjusted based on "desired" answer
    fn train(&mut self, inputs: &Vec<f32>, desired: i32) {
        // Guess the result
        let guess = self.feedforward(inputs);
        // Compute the factor for changing the weight based on the error
        // Error = desired output - guessed output
        // Note this can only be 0, -2, or 2
        // Multiply by learning constant
        let error = desired - guess;
        // Adjust weights based on weightChange * input
        for i in 0..self.weights.len() {
            self.weights[i] += self.c * error as f32 * inputs[i];
        }
    }

    // Guess -1 or 1 based on input values
    fn feedforward(&self, inputs: &Vec<f32>) -> i32 {
        // Sum all the values
        let mut sum = 0.0;
        for i in 0..self.weights.len() {
            sum += inputs[i] * self.weights[i];
        }
        self.activate(sum)
    }

    fn activate(&self, sum: f32) -> i32 {
        if sum > 0.0 {
            1
        } else {
            -1
        }
    }
}

struct Model {
    training: Vec<Trainer>, // A list of points we will use to "train" the perceptron
    ptron: Perceptron,      // A Perceptron object
    count: usize,           // We will train the perceptron with one "Point" object at a time
    x_min: f32,             // Coordinate space
    x_max: f32,             // Coordinate space
}

// The function to describe a line
fn f(x: f32) -> f32 {
    0.4 * x + 1.0
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();

    let x_min = -400.0;
    let y_min = -100.0;
    let x_max = 400.0;
    let y_max = 100.0;

    // The perceptron has 3 inputs -- x, y, and bias
    // Second value is "Learning Constant"
    let ptron = Perceptron::new(3, 0.00001); // Learning Constant is low just b/c it's fun to watch, this is not necessarily optimal

    // Create a random set of training points and calculate the "known" answer
    let training = (0..2000)
        .map(|_| {
            let x = random_range(x_min, x_max);
            let y = random_range(y_min, y_max);
            let answer = if y < f(x) { -1 } else { 1 };
            Trainer::new(x, y, answer)
        })
        .collect();

    Model {
        training,
        ptron,
        count: 0,
        x_min,
        x_max,
    }
}

fn update(_app: &App, m: &mut Model) {
    // Train the Perceptron with one "training" point at a time
    m.ptron
        .train(&m.training[m.count].inputs, m.training[m.count].answer);
    m.count = (m.count + 1) % m.training.len();
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // Draw the line
    let x1 = m.x_min;
    let y1 = f(x1);
    let x2 = m.x_max;
    let y2 = f(x2);
    draw.line()
        .start(pt2(x1, y1))
        .end(pt2(x2, y2))
        .color(GREY)
        .stroke_weight(4.0);

    // Draw the line based on the current weights
    // Formula is weights[0]*x + weights[1]*y + weights[2] = 0
    let weights = &m.ptron.weights;
    let y1 = (-weights[2] - weights[0] * x1) / weights[1];
    let y2 = (-weights[2] - weights[0] * x2) / weights[1];
    draw.line()
        .start(pt2(x1, y1))
        .end(pt2(x2, y2))
        .color(GREY)
        .stroke_weight(1.0);

    // Draw all the points based on what the Perceptron would "guess"
    // Does not use the "known" correct answer
    for i in 0..m.count {
        let guess = m.ptron.feedforward(&m.training[i].inputs);
        let x = m.training[i].inputs[0];
        let y = m.training[i].inputs[1];

        if guess > 0 {
            draw.ellipse().x_y(x, y).radius(4.0).stroke(BLACK).no_fill();
        } else {
            draw.ellipse().x_y(x, y).radius(4.0).color(BLACK);
        }
    }
}
