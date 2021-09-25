use nannou::prelude::*;

const ROWS: u32 = 10;
const SIZE: u32 = 100;
const COLS: u32 = 10;
const WIDTH: u32 = COLS * SIZE;
const HEIGHT: u32 = ROWS * SIZE;
const MARGIN: u32 = 150;
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
    let start_x = app.window_rect().top_left().x + MARGIN as f32;
    let start_y = app.window_rect().top_left().y * -1.0;

    let mut odd = false;
    for y in 1..ROWS {
        odd = !odd;

        let mut line = Vec::new();

        for x in 0..COLS {
            let step_x;
            if odd {
                step_x = SIZE / 2 + SIZE * x;
            } else {
                step_x = SIZE * x;
            }
            let next_x = step_x as f32 + start_x + random_range(-0.2, 0.2) * SIZE as f32;
            let next_y = (y * SIZE) as f32 + start_y + random_range(-0.2, 0.2) * SIZE as f32;
            let point = pt2(next_x, next_y);
            //println!("{}", point);
            draw.ellipse().x_y(next_x, next_y).radius(2.0).color(GRAY);
            line.push(point);
        }
        lines.push(line);
    }

    println!("{}", lines.len());

    odd = true;
    for y in 0..ROWS - 2 {
        odd = !odd;
        let mut dot_line = Vec::new();
        let current_line = &lines[y as usize];
        let next_line = &lines[(y + 1) as usize];
        for x in 0..COLS {
            let i = x as usize;
            if odd {
                dot_line.push(current_line[i]);
                dot_line.push(next_line[i]);
            } else {
                dot_line.push(next_line[i]);
                dot_line.push(current_line[i]);
            }
        }

        for i in 0..dot_line.len() - 2 {
            draw.tri()
                .color(get_gray_colour())
                .stroke_weight(1.0)
                .stroke_color(BLACK)
                .points(dot_line[i], dot_line[i + 1], dot_line[i + 2]);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn get_gray_colour() -> Rgb8 {
    let rand: u8 = random_range(30, 255);
    rgb8(rand, rand, rand)
}
