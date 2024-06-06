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

    // Use the mouse position to affect the frequency and amplitude.
    let hz = map_range(app.mouse().x, win.left(), win.right(), 0.0, 100.0);
    let amp = app.mouse().x;

    // Create an iterator yielding triangles for drawing a sine wave.
    let tris = (1..win.w() as usize)
        .flat_map(|i| {
            let l_fract = (i - 1) as f32 / win.w();
            let r_fract = i as f32 / win.w();

            // Map the vertices to the window.
            let l_x = map_range(l_fract, 0.0, 1.0, win.left(), win.right());
            let r_x = map_range(r_fract, 0.0, 1.0, win.left(), win.right());
            let l_y = (t * hz + l_fract * hz * TAU).sin() * amp;
            let r_y = (t * hz + r_fract * hz * TAU).sin() * amp;

            // Produce this slice of the triangle as a quad.
            let a = pt2(l_x, l_y);
            let b = pt2(r_x, r_y);
            let c = pt2(r_x, 0.0);
            let d = pt2(l_x, 0.0);
            geom::Quad([a, b, c, d]).triangles_iter()
        })
        .map(|tri| {
            // Color the vertices based on their amplitude.
            tri.map_vertices(|v| {
                let y_fract = map_range(v.y.abs(), 0.0, win.top(), 0.0, 1.0);
                let color = Color::srgba(y_fract, 1.0 - y_fract, 1.0 - y_fract, 1.0);
                (v.extend(0.0), color)
            })
        });

    // Draw the mesh!
    draw.mesh().tris_colored(tris);



}
