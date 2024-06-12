// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-5: Simple Harmonic Motion
use nannou::prelude::*;

fn main() {
    nannou::sketch(view).size(640, 360).run();
}

fn view(app: &App) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let period = 0.2;
    let amplitude = 300.0;
    // Calculating horizontal position according to formula for simple harmonic motion
    let two_pi = std::f64::consts::PI * 2.0;
    let x = amplitude * (two_pi * app.duration.since_start.secs() * period).sin() as f32;

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(pt2(x, 0.0))
        .rgb(0.0, 0.0, 0.0)
        .stroke_weight(2.0);

    draw.ellipse()
        .x_y(x, 0.0)
        .w_h(50.0, 50.0)
        .rgba(0.5, 0.5, 0.5, 1.0)
        .stroke(BLACK)
        .stroke_weight(2.0);
}
