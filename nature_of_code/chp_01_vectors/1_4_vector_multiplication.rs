// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-4: Vector Multiplication
use nannou::prelude::*;

fn main() {
    nannou::sketch(view).size(640, 360).run();
}

fn view(app: &App) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let mut mouse = vec2(app.mouse.x, app.mouse.y);
    let center = vec2(0.0, 0.0);

    // Multiplying a vector! The vector is now half its original size (multilies by 0.5)
    mouse *= 0.5;

    draw.line().weight(2.0).color(BLACK).points(center, mouse);



}
