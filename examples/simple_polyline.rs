extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::view(view);
}

fn view(app: &App, frame: Frame) -> Frame {
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

    // Create an iterator for each point in the polyline.
    let points = (0..n_points)
        .map(|i| {
            let x = map_range(i, 0, n_points-1, win.left(), win.right());
            let fract = i as f32 / n_points as f32;
            let amp = (t + fract * hz * TAU).sin();
            let y = map_range(amp, -1.0, 1.0, win.bottom() * 0.75, win.top() * 0.75);
            pt2(x, y)
        });

    // Get the miter vertices for the polyline and colour them.
    let vs = geom::line::join::miter::vertices(points, half_thickness)
        .enumerate()
        .flat_map(|(i, [nl, nr])| {
            let fract = i as f32 / n_points as f32;
            let r = (t + fract) % 1.0;
            let g = (t + 1.0 - fract) % 1.0;
            let b = (t + 0.5 + fract) % 1.0;
            let rgba = nannou::color::Rgba::new(r, g, b, 1.0);
            Some(geom::vertex::Rgba(nr, rgba))
                .into_iter()
                .chain(Some(geom::vertex::Rgba(nl, rgba)))
        });

    // Get the indices for triangulating the polyline.
    let is = geom::line::join::miter::triangle_indices(n_points);

    // Draw the polyline using a mesh.
    draw.mesh().indexed(vs, is);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
