// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 3-10: OOP Wave
extern crate nannou;

use nannou::prelude::*;
use nannou::app::Draw;
use nannou::geom::rect::Rect;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
    wave0: Wave,
    wave1: Wave,
}

struct Wave {
    x_spacing: f32, // How far apart should each horizontal position be spaced
    w: f32, // Width of entire wave
    origin: Vector2<f32>, // Where does the wave's first point start
    theta: f32, // Start angle at 0
    amplitude: f32, // Height of the wave
    period: f32, // How many pixels before the wave repeats
    dx: f32, // Value for incementing X, to be calculated as a function of period and x_spacing
    y_values: Vec<f32>, // Using a vector to store the height values for the wave (not entirely necessary)
}

impl Wave {
    fn new(o: Vector2<f32>, w: f32, a: f32, p: f32) -> Self {
        let origin = o;
        let x_spacing = 8.0 as f32;
        let theta = 0.0 as f32;
        let w = w;
        let period = p;
        let amplitude = a;
        let dx = (((std::f32::consts::PI * 2.0) / period) * x_spacing) as f32;
        let y_values = (0..(w / x_spacing) as i32).map(|_| 0.0).collect();
        Wave { origin, x_spacing, theta, w, period, amplitude, dx, y_values}
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
                .x_y(self.origin.x + x as f32 * self.x_spacing, self.origin.y + self.y_values[x])
                .w_h(48.0, 48.0)
                .rgba(0.0, 0.0, 0.0, 0.2);
        }
    }
}

fn model(app: &App) -> Model {
    let window = app.new_window().with_dimensions(750,200).build().unwrap();
    let wave0 = Wave::new(Vector2::new(-325.0,25.0), 100.0, 20.0, 500.0);
    let wave1 = Wave::new(Vector2::new(-75.0,0.0), 300.0, 40.0, 220.0);
    Model { window, wave0, wave1 }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            match event {
                // KEY EVENTS
                KeyPressed(_key) => {}

                // MOUSE EVENTS
                MouseReleased(_button) => {}

                _other => (),
            }
        }
        // update gets called just before view every frame
        Event::Update(_dt) => {
            // Update waves
            m.wave0.calculate();
            m.wave1.calculate();
        }
        _ => (),
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("OOP Wave");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    // display waves
    m.wave0.display(&draw);
    m.wave1.display(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
