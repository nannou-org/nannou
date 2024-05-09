// This example is a copy of the `simple_draw.rs` example, but captures each frame and writes them
// as a PNG image file to `/<path_to_nannou>/nannou/simple_capture/<frame_number>.png`

use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    let draw = app.draw();

    draw.background().color(CORNFLOWERBLUE);

    let win = app.window_rect();
    draw.tri()
        .points(win.bottom_left(), win.top_left(), win.top_right())
        .color(VIOLET);

    let t = app.elapsed_frames() as f32 / 60.0;
    draw.ellipse()
        .x_y(app.mouse().x * t.cos(), app.mouse().y)
        .radius(win.w() * 0.125 * t.sin())
        .color(RED);

    draw.line()
        .weight(10.0 + (t.sin() * 0.5 + 0.5) * 90.0)
        .caps_round()
        .color(PALEGOLDENROD)
        .points(win.top_left() * t.sin(), win.bottom_right() * t.cos());

    draw.quad()
        .x_y(-app.mouse().x, app.mouse().y)
        .color(DARKGREEN)
        .rotate(t);

    draw.rect()
        .x_y(app.mouse().y, app.mouse().x)
        .w(app.mouse().x * 0.25)
        .hsv(t, 1.0, 1.0);



    // Capture the frame!
    let file_path = captured_frame_path(app, &frame);
    app.main_window().capture_frame(file_path);
}

fn captured_frame_path(app: &App, frame: &Frame) -> std::path::PathBuf {
    // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `/<path_to_nannou>/nannou/simple_capture`.
        .join(app.exe_name().unwrap())
        // Name each file after the number of the frame.
        .join(format!("{:03}", app.elapsed_frames()))
        // The extension will be PNG. We also support tiff, bmp, gif, jpeg, webp and some others.
        .with_extension("png")
}
