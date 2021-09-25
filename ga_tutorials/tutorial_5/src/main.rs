use nannou::prelude::*;

const ROWS: u32 = 20;
const SIZE: u32 = 32;
const COLS: u32 = 20;
const WIDTH: u32 = COLS * SIZE;
const HEIGHT: u32 = ROWS * SIZE;
const MARGIN: u32 = 25;
fn main() {
    nannou::sketch(view)
        .size(WIDTH + 2 * MARGIN, HEIGHT + 2 * MARGIN)
        .run()
}

fn view(app: &App, frame: Frame) {
    app.set_loop_mode(LoopMode::loop_once());

    let draw = app.draw();
    draw.background().color(WHITE);

    let mut lines = Vec::new();
    let start_x = app.window_rect().bottom_left().x + MARGIN as f32;
    let start_y = app.window_rect().bottom_left().y + MARGIN as f32;

    for y in 0..ROWS {
        let mut line = Vec::new();
        for x in 1..COLS {
            let step_x: Vec<f32>;
            if y > ROWS / 3 * 2 {
                step_x = [0.5].to_vec();
            } else if y > ROWS / 3 {
                step_x = [0.2, 0.8].to_vec();
            } else {
                step_x = [0.1, 0.5, 0.9].to_vec();
            }

            for i in step_x {
                let next_x = start_x + (x * SIZE) as f32 + (i - 0.5) * SIZE as f32;
                let next_y = start_y + (y * SIZE) as f32;
                line.push(pt2(next_x, next_y));
            }
        }

        lines.push(line);
    }

    for y in 0..lines.len() - 1 {
        let current_line = &lines[y as usize];
        let next_line = &lines[(y + 1) as usize];
        if current_line.len() == next_line.len() {
            for x in 0..current_line.len() {
                draw_line(&draw, next_line[x].x, next_line[x].y)
            }
        } else if current_line.len() > next_line.len() {
            for x in 0..current_line.len() {
                draw_line(&draw, current_line[x].x, current_line[x].y + SIZE as f32);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn draw_line(draw: &Draw, x: f32, y: f32) {
    draw.rect()
        .w_h(5.0, SIZE as f32)
        .color(BLACK)
        .rotate(random_range(2.0, 7.0))
        .x_y(x, y);
}
