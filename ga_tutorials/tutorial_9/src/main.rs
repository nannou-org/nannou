use nannou::prelude::*;
use std::f64::consts::PI;

const ROWS: u32 = 23;
const SIZE: u32 = 32;
const COLS: u32 = 16;
const WIDTH: u32 = COLS * SIZE + MARGIN;
const HEIGHT: u32 = ROWS * SIZE + MARGIN;
const MARGIN: u32 = 25;
const DAYS: u32 = 356;

struct Line {
    x: f32,
    y: f32,
    scale: f32,
    rotate: f32
}

fn main() {
    nannou::sketch(view).size(WIDTH, HEIGHT).run()
}

fn view(app: &App, frame: Frame) {
    app.set_loop_mode(LoopMode::loop_once());

    let draw = app.draw();
    draw.background().color(WHITE);

    let start_x = app.window_rect().bottom_left().x + MARGIN as f32;
    let start_y = app.window_rect().bottom_left().y + MARGIN as f32;


    for i in 0..DAYS {
        let col = i / ROWS;
        let row = i % ROWS;

        let x = start_x + (col * SIZE as u32) as f32;
        let y = start_y + (row * SIZE as u32) as f32;


        let phi = (i as f64 / 365.0) * PI;
        let roate = phi.sin() * PI * 0.45 + 0.85;
        let scale = phi.cos().abs()*2.0+1.0;
        let line = Line {
            x: x,
            y: y,
            scale: scale as f32,
            rotate: roate as f32
        };
        draw_line(&draw, line);
    }
    draw.to_frame(app, &frame).unwrap();
}

fn draw_line(draw: &Draw, line: Line) {
    draw.scale_y(-1.0)
        .rect()
        .w_h(5.0 * line.scale, SIZE as f32)
        .color(BLACK)
        .rotate(line.rotate)
        .x_y(line.x, line.y);
}
