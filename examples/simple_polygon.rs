extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::view(view);
}

fn view(app: &App, frame: Frame) -> Frame {
    // Begin drawing 
    let win = app.window_rect();
    let t = app.time;
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(BLACK);

    // Create an `ngon` of points.
    let n_points = 5;
    let radius = win.w().min(win.h()) * 0.25;
    let points = (0..n_points)
        .map(|i| {
            let fract = i as f32 / n_points as f32;
            let phase = fract;
            let x = radius * (TAU * phase).cos();
            let y = radius * (TAU * phase).sin();
            pt3(x, y, 0.0)
        });
    draw.polygon()
        .points(points)
        .x(-win.w() * 0.25)
        .color(WHITE)
        .rotate(-t * 0.1);

    // Do the same, but give each point a unique colour.
    let n_points = 7;
    let colored_points = (0..n_points)
        .map(|i| {
            let fract = i as f32 / n_points as f32;
            let phase = fract;
            let x = radius * (TAU * phase).cos();
            let y = radius * (TAU * phase).sin();
            let r = fract;
            let g = 1.0 - fract;
            let b = (0.5 + fract) % 1.0;
            let a = 1.0;
            let color = Rgba::new(r, g, b, a);
            (pt3(x, y, 0.0), color)
        });
    draw.polygon()
        .colored_points(colored_points)
        .x(win.w() * 0.25)
        .rotate(t * 0.2);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
