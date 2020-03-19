//! A small demonstration of how to use the `draw.scale(s)` and `draw.rotate(r)` transform methods
//! to transform the drawing context. This can be useful when applying transformations to whole
//! groups of drawings, rather than one at a time. Note that `scale` and `rotate` are only two of
//! many transform methods. See the link below to find a whole suite of interesting draw methods:
//!
//! https://docs.rs/nannou/latest/nannou/draw/struct.Draw.html

use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App, frame: Frame) {
    frame.clear(BLACK);

    // Begin drawing
    let draw = app.draw();
    let w = app.window_rect();

    // Slowly turn the whole drawing inside out.
    let draw = draw.scale((app.time * 0.1).cos());

    // Draw a tunnel of rectangles.
    let max_side = w.right().max(w.top());
    let gap = 30.0;
    let len = (max_side / gap) as u32;
    for i in 1..=len {
        let f = i as f32 / len as f32;

        // Return a new rotated draw instance.
        // This will rotate both the rect and text around the origin.
        let rotate = (app.time * 0.5).sin() * (app.time * 0.25 + f * PI * 2.0).cos();
        let draw = draw.rotate(rotate);

        let hue = app.time + f * 2.0 * PI;
        let color = hsl(hue, 0.5, 0.5);
        let rect_scale = f.powi(2) * max_side * 2.0;
        draw.scale(rect_scale)
            .rect()
            .w_h(1.0, 1.0)
            .stroke(color)
            .stroke_weight(1.0 / len as f32)
            .no_fill()
            .w_h(1.0, 1.0);

        let text_scale = rect_scale * 0.001;
        draw.scale(text_scale)
            .text("woah")
            .wh(w.wh() * 0.8)
            .align_text_bottom()
            .color(color)
            .font_size(96);
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
