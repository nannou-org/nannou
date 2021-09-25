use nannou::{geom::Path, prelude::*};

const ROWS: u32 = 20;
const SIZE: u32 = 50;
const COLS: u32 = 25;
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
    let start_x = app.window_rect().top_left().x + 50.0;
    let start_y = app.window_rect().top_left().y * -1.0;
    let max_y = app.window_rect().top_right().y - 15.0;

    let step_x = app.window_rect().w() / (COLS as f32);
    for y in 0..ROWS {
        let mut line = Vec::new();

        for x in 0..COLS {
            let distance = (x as f32 * step_x) - (WIDTH / 2) as f32;
            let center_distance = WIDTH as i16 / 2 - 50 - distance.abs() as i16;
            let variance = std::cmp::max(center_distance, 0);
            let random = random_range(0.0, 1.0) * variance as f32 / 2.0 * 1.0;

            let next_x = start_x + (x as f32) * step_x;
            let mut next_y = start_y + (y * SIZE) as f32 + random;
            if next_y > max_y {
                next_y = max_y;
            }
            line.push(pt2(next_x, next_y));
        }
        lines.push(line);
    }

    for y in 0..ROWS {
        let line = lines.get(y as usize).unwrap();
        let mut path = Path::builder().move_to(line[0]);

        for x in 5..COLS - 3 {
            let i = x as usize;
            let xc = (line[i].x + line[i + 1 as usize].x) / 2.0;
            let yc = (line[i].y + line[i + 1 as usize].y) / 2.0;
            path = path.quadratic_bezier_to(line[i], pt2(xc, yc));
        }
        path = path.quadratic_bezier_to(line[line.len() - 2], line[line.len() - 1]);

        draw.polyline()
            .stroke_weight(3.0)
            .color(BLACK)
            .caps_round()
            .events(path.build().iter());
    }

    draw.to_frame(app, &frame).unwrap();
}
