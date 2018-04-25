// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 5-07: Oscillating Objects 
extern crate nannou;

use nannou::prelude::*;
use nannou::app::Draw;
use nannou::geom::rect::Rect;
use nannou::math::map_range;
use nannou::rand::random;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
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
        let angle = Vector2::new(0.0, 0.0);
        let velocity = Vector2::new(random::<f32>() * 0.1 - 0.05, random::<f32>() * 0.1 - 0.05);
        let rand_amp_x = map_range(random(), 0.0, 1.0, 20.0, rect.right());
        let rand_amp_y = map_range(random(), 0.0, 1.0, 20.0, rect.top());
        let amplitude = Vector2::new(rand_amp_x, rand_amp_y);
        Oscillator { angle, velocity, amplitude }
    }

    fn oscillate(&mut self) {
        self.angle += self.velocity;
    }

    fn display(&self, draw: &Draw) {
        let x = self.angle.x.sin() * self.amplitude.x;
        let y = self.angle.y.sin() * self.amplitude.y;

        draw.ellipse()
            .x_y(x, y)
            .w_h(32.0, 32.0)
            .rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    let rect = Rect::from_wh(Vector2::new(640.0,360.0));
    let window = app.new_window().with_dimensions(rect.w() as u32, rect.h()as u32).build().unwrap();
    //let oscillators = vec![Oscillator::new(app.window.rect()); 10];
    let oscillators = (0..10).map(|_| Oscillator::new(rect)).collect();
    Model { window, oscillators }
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
            for osc in &mut m.oscillators {
                osc.oscillate();
            }
        }
        _ => (),
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("oscillating object");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgba(1.0,1.0,1.0,1.0);

    for osc in &m.oscillators {
        osc.display(&draw);
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
