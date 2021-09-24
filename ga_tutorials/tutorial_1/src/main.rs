use nannou::prelude::*;

const ROWS: u32 = 20;
const COLS: u32 = 20;
const SIZE: u32 = 50;
const WIDTH: u32 = COLS * SIZE;
const HEIGHT: u32 = ROWS * SIZE;
fn main() {
    nannou::sketch(view).size(WIDTH, HEIGHT).run()
}

fn view(app: &App, frame: Frame) {
    app.set_loop_mode(LoopMode::loop_once());

    let draw = app.draw();
    draw.background().color(WHITE);

    draw_line(&draw, SIZE as f32, 0.1);

    draw.to_frame(app, &frame).unwrap();
}



fn draw_line(draw: &Draw, step: f32, weight: f32) {
    let gdraw = draw
        .scale(step)
        .scale_y(-1.0)
        .x_y(COLS as f32 / -2.0, ROWS as f32 / -2.0);

    for y in 0..ROWS {
        for x in 0..COLS {
            let cdraw = gdraw.x_y(x as f32, y as f32);
            let (start, end) = get_point();
            cdraw.line().color(BLACK).weight(weight).points(start, end);
        }
    }
}

fn get_point() -> (Point2, Point2) {
    let r = random_range(0.0, 1.0);
    if r > 0.5 {
        return (pt2(0.0, 1.0), pt2(1.0, 0.0));
    } else {
        return (pt2(0.0, 0.0), pt2(1.0, 1.0));
    }
}
