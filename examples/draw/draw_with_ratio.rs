use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    // Get the window rect and ensure it's ratio is 16/9.
    let window_rect = app.window_rect().with_ratio(16.0 / 9.0);

    // Draw the boundaries.
    draw.rect()
        .xy(window_rect.xy())
        .wh(window_rect.wh())
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(2.0);

    // Example from draw_arrow.rs
    let r = window_rect;
    for r in r.subdivisions_iter() {
        for r in r.subdivisions_iter() {
            for r in r.subdivisions_iter() {
                let side = r.w().min(r.h());
                let start = r.xy();
                let start_to_mouse = app.mouse.position() - start;
                let target_mag = start_to_mouse.length().min(side * 0.5);
                let end = start + start_to_mouse.normalize() * target_mag;
                draw.arrow().weight(5.0).points(start, end);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
