use nannou::prelude::*;
use std::cell::Cell;

const STEP: usize = 7;
const SIZE: usize = 80;
const LINE_WIDTH: f32 = 8.0;
const MARGIN: usize = 10;
const WIDTH: usize = STEP * SIZE + 2 * MARGIN;
const HEIGHT: usize = STEP * SIZE + 2 * MARGIN;

struct Square {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    colour: Cell<Rgb8>,
}

impl Square {
    fn new() -> Square {
        Square {
            x: 0.0,
            y: 0.0,
            w: (STEP * SIZE) as f32,
            h: (STEP * SIZE) as f32,
            colour: Cell::new(WHITE),
        }
    }

    fn copy(&self) -> Square {
        Square {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
            colour: Cell::new(self.colour.get()),
        }
    }
}

fn main() {
    nannou::sketch(view).size(WIDTH as u32, HEIGHT as u32).run()
}

fn view(app: &App, frame: Frame) {
    app.set_loop_mode(LoopMode::loop_once());

    let draw = app.draw();
    draw.background().color(SNOW);

    let mut squares = Vec::new();
    squares.push(Square::new());

    for i in 1..STEP {
        let next = (i * SIZE) as f32;
        split_square(&mut squares, pt2(0.0, next));
        split_square(&mut squares, pt2(next, 0.0));
    }

    let colours = [rgb8(212, 9, 32), rgb8(19, 86, 162), rgb8(247, 216, 66)];

    for colour in colours {
        squares[random_range(0, squares.len() - 1)]
            .colour
            .set(colour);
    }
    for square in squares {
        draw_square(
            &draw,
            &square,
            app.window_rect().bottom_left().x + MARGIN as f32,
            app.window_rect().bottom_left().y,
        );
    }

    draw.to_frame(app, &frame).unwrap();
}

fn draw_square(draw: &Draw, square: &Square, start_x: f32, start_y: f32) {
    let x = square.x + start_x + square.w / 2.0;
    let y = square.y + start_y + square.h / 2.0;
    draw.rect()
        .stroke_color(BLACK)
        .stroke_weight(LINE_WIDTH)
        .color(square.colour.get())
        .w_h(square.w, square.h)
        .x_y(x, y);
}

fn split_square(sqares: &mut Vec<Square>, point: Point2) {
    let x = point.x;
    let y = point.y;
    for i in (0..sqares.len()).rev() {
        let square = sqares[i].copy();
        if x > 0.0 && x > square.x && x < (square.x + square.w) {
            let r = random_range(0.0, 1.0);
            if r > 0.5 {
                sqares.remove(i);
                split_x(sqares, &square, x);
            }
        }

        if y > 0.0 && y > square.y && y < (square.y + square.h) {
            let r = random_range(0.0, 1.0);
            if r > 0.5 {
                sqares.remove(i);
                split_y(sqares, &square, y);
            }
        }
    }
}

fn split_x(sqares: &mut Vec<Square>, square: &Square, split: f32) {
    let square_a = Square {
        x: square.x,
        y: square.y,
        w: square.w - (square.w - split + square.x),
        h: square.h,
        colour: Cell::new(square.colour.get()),
    };

    let square_b = Square {
        x: split,
        y: square.y,
        w: square.w - split + square.x,
        h: square.h,
        colour: Cell::new(square.colour.get()),
    };
    sqares.push(square_a);
    sqares.push(square_b);
}

fn split_y(sqares: &mut Vec<Square>, square: &Square, split: f32) {
    let square_a = Square {
        x: square.x,
        y: square.y,
        w: square.w,
        h: square.h - (square.h - split + square.y),
        colour: Cell::new(square.colour.get()),
    };

    let square_b = Square {
        x: square.x,
        y: split,
        w: square.w,
        h: square.h - split + square.y,
        colour: Cell::new(square.colour.get()),
    };
    sqares.push(square_a);
    sqares.push(square_b);
}
