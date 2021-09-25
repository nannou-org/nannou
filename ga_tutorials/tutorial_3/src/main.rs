// adapte the code from https://github.com/sidwellr/schotter/blob/main/schotter1/src/main.rs
// I really encourage follow the tutorial

use nannou::prelude::*;

const ROWS: u32 = 22;
const COLS: u32 = 12;
const SIZE: u32 = 30;
const LINE_WIDTH: f32 = 0.06;
const MARGIN: u32 = 35;
const WIDTH: u32 = COLS * SIZE + 2 * MARGIN;
const HEIGHT: u32 = ROWS * SIZE + 2 * MARGIN;

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
            let factor = y as f32 / ROWS as f32;
            let x_offset = factor * random_range(-0.5, 0.5);
            let y_offset = factor * random_range(-0.5, 0.5);
            let rotation = factor * random_range(-PI / 4.0, PI / 4.0);
            cdraw
                .rect()
                .no_fill()
                .stroke(BLACK)
                .stroke_weight(LINE_WIDTH)
                .w_h(1.0, 1.0)
                .x_y(x_offset, y_offset)
                .rotate(rotation);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}
