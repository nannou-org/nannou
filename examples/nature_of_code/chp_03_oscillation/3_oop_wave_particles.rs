// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// example 3-x: OOP Wave Particle
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    wave0: Wave,
    wave1: Wave,
}

struct Particle {
    position: Point2,
}

struct Wave {
    x_spacing: f32,      // How far apart should each horizontal position be spaced
    origin: Point2,      // Where does the wave's first point start
    theta: f32,          // Start angle at 0
    amplitude: f32,      // Height of the wave
    dx: f32, // Value for incementing X, to be calculated as a function of period and x_spacing
    _y_values: Vec<f32>, // Using a vector to store the height values for the wave (not entirely necessary)
    particles: Vec<Particle>,
}

impl Particle {
    fn new() -> Self {
        let position = pt2(0.0, 0.0);
        Particle { position }
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    fn display(&self, draw: &app::Draw) {
        let random_color = random();
        draw.ellipse()
            .xy(self.position)
            .w_h(16.0, 16.0)
            .rgb(random_color, random_color, random_color)
            .stroke(BLACK);
    }
}
impl Wave {
    fn new(o: Point2, w: f32, a: f32, p: f32) -> Self {
        let origin = o;
        let x_spacing = 8.0 as f32;
        let theta = 0.0 as f32;
        let w = w; // Width of entire wave
        let period = p; // How many pixels before the wave repeats
        let amplitude = a;
        let dx = (((std::f32::consts::PI * 2.0) / period) * x_spacing) as f32;
        let range = (w / x_spacing) as i32;
        let _y_values = (0..range).map(|_| 0.0).collect();
        let particles = (0..range).map(|_| Particle::new()).collect();
        Wave {
            origin,
            x_spacing,
            theta,
            amplitude,
            dx,
            _y_values,
            particles,
        }
    }

    fn calculate(&mut self) {
        // Increment theta (try different values for 'angular velocity' here
        self.theta += 0.02;

        // For every x values, calculate a y value with sine function
        let mut x = self.theta;
        for i in 0..self.particles.len() {
            self.particles[i].set_position(
                self.origin.x + i as f32 * self.x_spacing,
                self.origin.y + x.sin() * self.amplitude,
            );
            x += self.dx;
        }
    }

    fn display(&self, draw: &app::Draw) {
        // A simple way to draw the wave with an ellipse at each position
        for x in 0..self.particles.len() {
            self.particles[x].display(&draw);
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(750, 200)
        .view(view)
        .build()
        .unwrap();
    let wave0 = Wave::new(pt2(-325.0, 25.0), 100.0, 20.0, 500.0);
    let wave1 = Wave::new(pt2(-75.0, 0.0), 300.0, 40.0, 220.0);
    Model { wave0, wave1 }
}

fn update(_app: &App, m: &mut Model, _update: Update) {
    // Update waves
    m.wave0.calculate();
    m.wave1.calculate();
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // display waves
    m.wave0.display(&draw);
    m.wave1.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
