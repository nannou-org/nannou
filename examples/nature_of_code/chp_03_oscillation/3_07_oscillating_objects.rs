// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 5-07: Oscillating Objects
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    // An array of type Oscillator
    oscillators: Vec<Oscillator>,
}

#[derive(Clone)]
struct Oscillator {
    angle: Vector2<f32>,
    velocity: Vector2<f32>,
    amplitude: Vector2<f32>,
}

impl Oscillator {
    fn new(rect: Rect<f32>) -> Self {
        let angle = vec2(0.0, 0.0);
        let velocity = vec2(random_f32() * 0.1 - 0.05, random_f32() * 0.1 - 0.05);
        let rand_amp_x = random_range(20.0, rect.right());
        let rand_amp_y = random_range(20.0, rect.top());
        let amplitude = vec2(rand_amp_x, rand_amp_y);
        Oscillator {
            angle,
            velocity,
            amplitude,
        }
    }

    fn oscillate(&mut self) {
        self.angle += self.velocity;
    }

    fn display(&self, draw: &app::Draw) {
        let x = self.angle.x.sin() * self.amplitude.x;
        let y = self.angle.y.sin() * self.amplitude.y;

        draw.ellipse().x_y(x, y).w_h(32.0, 32.0).rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(640.0, 360.0);
    let _window = app.new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .build()
        .unwrap();
    //let oscillators = vec![Oscillator::new(app.window_rect()); 10];
    let oscillators = (0..10).map(|_| Oscillator::new(rect)).collect();
    Model { oscillators }
}

fn event(_app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        for osc in &mut m.oscillators {
            osc.oscillate();
        }
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().rgba(1.0, 1.0, 1.0, 1.0);

    for osc in &m.oscillators {
        osc.display(&draw);
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
