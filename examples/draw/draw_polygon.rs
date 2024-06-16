use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    // Begin drawing
    let win = app.window_rect();
    let t = app.elapsed_seconds();
    let draw = app.draw();

    // Clear the background to black.
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
        .color(WHITE)
        .rotate(-t * 0.1)
        .stroke(PINK)
        .stroke_weight(20.0)
        .join_round()
        .points(points);

    // Do the same, but give each point a unique colour.
    let n_points = 7;
    let points_colored = (0..n_points).map(|i| {
        let fract = i as f32 / n_points as f32;
        let phase = fract;
        let x = radius * (TAU * phase).cos();
        let y = radius * (TAU * phase).sin();
        let r = fract;
        let g = 1.0 - fract;
        let b = (0.5 + fract) % 1.0;
        (pt2(x, y), Color::srgb(r, g, b))
    });
    draw.polygon()
        .x(win.w() * 0.25)
        .rotate(t * 0.2)
        .points_colored(points_colored);
}
