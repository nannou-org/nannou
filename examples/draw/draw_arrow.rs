use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App, frame: Frame) {
    let draw = app.draw();
    let r = app.window_rect();
    draw.background().color(BLACK);

    for r in r.subdivisions_iter() {
        for r in r.subdivisions_iter() {
            for r in r.subdivisions_iter() {
                let side = r.w().min(r.h());
                let start = r.xy();
                let start_to_mouse = app.mouse.position() - start;
                let target_mag = start_to_mouse.magnitude().min(side * 0.5);
                let end = start + start_to_mouse.with_magnitude(target_mag);
                draw.arrow().weight(5.0).points(start, end);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
