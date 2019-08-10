use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: &Frame) {
    // Begin drawing
    let win = app.window_rect();
    let t = app.time;
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(BLACK);

    // Create an `ngon` of points.
    let n_points = 5;
    let radius = win.w().min(win.h()) * 0.25;
    let points = (0..n_points).map(|i| {
        let fract = i as f32 / n_points as f32;
        let phase = fract;
        let x = radius * (TAU * phase).cos();
        let y = radius * (TAU * phase).sin();
        pt2(x, y)
    });
    draw.polygon()
        .x(-win.w() * 0.25)
        .color(LIGHTGREEN)
        .rotate(-t * 0.1)
        .points(points);

    // Do the same, but give each point a unique colour.
    let n_points = 7;
    let points = (0..n_points).map(|i| {
        let fract = i as f32 / n_points as f32;
        let phase = fract;
        let x = radius * (TAU * phase).cos();
        let y = radius * (TAU * phase).sin();
        pt2(x, y)
    });
    draw.polygon()
        .stroke(CORNFLOWERBLUE)
        .stroke_weight(20.0)
        .join_round()
        .color(CORAL)
        .x(win.w() * 0.25)
        .rotate(t * 0.2)
        .points(points);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
