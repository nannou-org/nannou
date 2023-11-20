use nannou::draw::renderer::Instance;
use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App, frame: Frame) {
    // Begin drawing
    let win = app.window_rect();
    let t = app.time;
    let draw = app.draw();
    let radius = win.w().min(win.h()) * 0.25;
    let dim_x = win.w() / 2.0;
    let n_instances = 10;



    // Clear the background to blue.
    draw.background().color(BLACK);

    let instances = (0 .. n_instances).map(|row|{
        Instance {transform: Mat4::from_translation(Vec3::new(-dim_x + row as f32 * (2.0 * dim_x / n_instances as f32), 0.0, 0.0))}
    }).collect();

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
        (pt2(x, y), rgb(r, g, b))
    });

    draw.instances(instances).polygon()
        .rotate(t * 0.2)
        .points_colored(points_colored);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
