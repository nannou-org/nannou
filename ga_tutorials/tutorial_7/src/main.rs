use nannou::prelude::*;

const ROWS: u32 = 7;
const COLS: u32 = 7;
const SIZE: u32 = 150;
const LINE_WIDTH: f32 = 0.03;
const MARGIN: u32 = 35;
const WIDTH: u32 = COLS * SIZE + 2 * MARGIN;
const HEIGHT: u32 = ROWS * SIZE + 2 * MARGIN;
const STEP_SIZE: f32 = 1.0 / 7.0;

fn main() {
    nannou::sketch(view).size(WIDTH, HEIGHT).run()
}

fn view(app: &App, frame: Frame) {
    app.set_loop_mode(LoopMode::loop_once());

    let draw = app.draw();
    let gdraw = draw
        .scale(SIZE as f32)
        .scale_y(-1.0)
        .x_y(COLS as f32 / -2.0 + 0.5, ROWS as f32 / -2.0 + 0.5);

    draw.background().color(SNOW);

    for y in 0..ROWS {
        for x in 0..COLS {
            let cdraw = gdraw.x_y(x as f32, y as f32);
            let step = random_range(2, 7);
            let direction = random_range(-0.3, 0.3);
            draw_rect(&cdraw, 0.0, 0.0, 1.0, step, direction);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn draw_rect(draw: &Draw, x: f32, y: f32, h: f32, step: i32, direction: f32) {
    draw.rect()
        .no_fill()
        .stroke(BLACK)
        .stroke_weight(LINE_WIDTH)
        .w_h(h, h)
        .x_y(x, y);

    if step < 0 {
        return;
    }

    let next_size = step as f32 * STEP_SIZE;
    let next_x = x + (h - next_size) / 2.0 * direction;
    let next_y = y + (h - next_size) / 2.0 * direction;
    draw_rect(draw, next_x, next_y, next_size, step - 1, direction);
}
