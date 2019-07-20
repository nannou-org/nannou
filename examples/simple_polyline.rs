use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background.
    draw.background().color(BLACK);

    let win = app.window_rect();
    let t = app.time;

    // Decide on a number of points and a thickness.
    let n_points = 10;
    let half_thickness = 4.0;
    let hz = ((app.mouse.x + win.right()) / win.w()).powi(4) * 1000.0;
    let vertices = (0..n_points)
        // A sine wave mapped to the range of the window.
        .map(|i| {
            let x = map_range(i, 0, n_points - 1, win.left(), win.right());
            let fract = i as f32 / n_points as f32;
            let amp = (t + fract * hz * TAU).sin();
            let y = map_range(amp, -1.0, 1.0, win.bottom() * 0.75, win.top() * 0.75);
            pt2(x, y)
        })
        .enumerate()
        // Colour each vertex uniquely based on its index.
        .map(|(i, p)| {
            let fract = i as f32 / n_points as f32;
            let r = (t + fract) % 1.0;
            let g = (t + 1.0 - fract) % 1.0;
            let b = (t + 0.5 + fract) % 1.0;
            let rgba = nannou::color::Rgba::new(r, g, b, 1.0);
            geom::vertex::Rgba(p, rgba)
        });

    // Draw the polyline.
    draw.polyline().vertices(half_thickness, vertices);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
