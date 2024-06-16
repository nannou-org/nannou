// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 3-10: OOP Wave
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    wave0: Wave,
    wave1: Wave,
}

struct Wave {
    x_spacing: f32,     // How far apart should each horizontal position be spaced
    _w: f32,            // Width of entire wave
    origin: Point2,     // Where does the wave's first point start
    theta: f32,         // Start angle at 0
    amplitude: f32,     // Height of the wave
    _period: f32,       // How many pixels before the wave repeats
    dx: f32, // Value for incementing X, to be calculated as a function of period and x_spacing
    y_values: Vec<f32>, // Using a vector to store the height values for the wave (not entirely necessary)
}

impl Wave {
    fn new(o: Point2, w: f32, a: f32, p: f32) -> Self {
        let origin = o;
        let x_spacing = 8.0_f32;
        let theta = 0.0_f32;
        let _w = w;
        let _period = p;
        let amplitude = a;
        let dx = ((PI * 2.0) / _period) * x_spacing;
        let y_values = (0..(w / x_spacing) as i32).map(|_| 0.0).collect();
        Wave {
            origin,
            x_spacing,
            theta,
            _w,
            _period,
            amplitude,
            dx,
            y_values,
        }
    }

    fn calculate(&mut self) {
        // Increment theta (try different values for 'angular velocity' here
        self.theta += 0.02;

        // For every x values, calculate a y value with sine function
        let mut x = self.theta;
        for i in 0..self.y_values.len() {
            self.y_values[i] = x.sin() * self.amplitude;
            x += self.dx;
        }
    }

    fn display(&self, draw: &Draw) {
        // A simple way to draw the wave with an ellipse at each position
        for x in 0..self.y_values.len() {
            draw.ellipse()
                .x_y(
                    self.origin.x + x as f32 * self.x_spacing,
                    self.origin.y + self.y_values[x],
                )
                .w_h(48.0, 48.0)
                .rgba(0.0, 0.0, 0.0, 0.2)
                .stroke(BLACK);
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window().size(750, 200).view(view).build();
    let wave0 = Wave::new(pt2(-325.0, 25.0), 100.0, 20.0, 500.0);
    let wave1 = Wave::new(pt2(-75.0, 0.0), 300.0, 40.0, 220.0);
    Model { wave0, wave1 }
}

fn update(_app: &App, m: &mut Model) {
    // Update waves
    m.wave0.calculate();
    m.wave1.calculate();
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // display waves
    m.wave0.display(&draw);
    m.wave1.display(&draw);
}
