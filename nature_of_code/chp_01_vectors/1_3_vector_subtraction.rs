// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-3: Vector Subtraction
use nannou::prelude::*;

fn main() {
    nannou::sketch(view).size(640, 360).run();
}

fn view(app: &App) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let mut mouse = app.mouse();
    let center = vec2(0.0, 0.0);
    mouse -= center;

    draw.line().weight(2.0).color(BLACK).points(center, mouse);
}
