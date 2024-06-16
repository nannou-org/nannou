// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 5-07: Oscillating Objects
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    // An array of type Oscillator
    oscillators: Vec<Oscillator>,
}

#[derive(Clone)]
struct Oscillator {
    angle: Vec2,
    velocity: Vec2,
    amplitude: Vec2,
}

impl Oscillator {
    fn new(rect: geom::Rect) -> Self {
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

    fn display(&self, draw: &Draw) {
        let x = self.angle.x.sin() * self.amplitude.x;
        let y = self.angle.y.sin() * self.amplitude.y;

        draw.line()
            .start(pt2(0.0, 0.0))
            .end(pt2(x, y))
            .rgb(0.0, 0.0, 0.0)
            .stroke_weight(2.0);

        draw.ellipse()
            .x_y(x, y)
            .w_h(32.0, 32.0)
            .rgba(0.5, 0.5, 0.5, 0.5)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }
}

fn model(app: &App) -> Model {
    let rect = geom::Rect::from_w_h(640.0, 360.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .view(view)
        .build();
    //let oscillators = vec![Oscillator::new(app.window_rect()); 10];
    let oscillators = (0..10).map(|_| Oscillator::new(rect)).collect();
    Model { oscillators }
}

fn update(_app: &App, m: &mut Model) {
    for osc in &mut m.oscillators {
        osc.oscillate();
    }
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().rgba(1.0, 1.0, 1.0, 1.0);

    for osc in &m.oscillators {
        osc.display(&draw);
    }
}
