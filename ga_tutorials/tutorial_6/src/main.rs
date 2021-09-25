use nannou::prelude::*;
use std::cell::Cell;

const MIN_RADIUS: f32 = 2.0;
const MAX_RADIUS: usize = 100;
const TOTAL_CIRCLE: u16 = 2500;
const CREATE_CIRCLE_ATTEMPTS: u16 = 500;
const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

struct Circle {
    x: f32,
    y: f32,
    r: Cell<f32>,
}

impl Circle {
    fn new() -> Circle {
        let min_x = -1.0 * (WIDTH as f32);
        let max_x = WIDTH as f32;
        let min_y = -1.0 * (HEIGHT as f32);
        let max_y = HEIGHT as f32;
        Circle {
            x: random_range(min_x, max_x),
            y: random_range(min_y, max_y),
            r: Cell::new(MIN_RADIUS),
        }
    }

    fn increase(&self) {
        self.r.set(self.r.get() + 1.0);
    }

    fn decrease(&self) {
        self.r.set(self.r.get() - 1.0);
    }
}

fn main() {
    nannou::sketch(view).size(WIDTH, HEIGHT).run()
}

fn view(app: &App, frame: Frame) {
    app.set_loop_mode(LoopMode::loop_once());

    let draw = app.draw();
    draw.background().color(WHITE);

    let mut circles: Vec<Circle> = Vec::new();

    for _ in 0..TOTAL_CIRCLE {
        create_draw_circle(&mut circles);
    }

    let mut i = 0;
    for c in circles {
        draw.ellipse()
            .stroke_weight(2.0)
            .no_fill()
            .stroke_color(BLACK)
            .x_y(c.x, c.y)
            .radius(c.r.get());
        i = i + 1;
    }

    draw.to_frame(app, &frame).unwrap();
}

fn create_draw_circle(circles: &mut Vec<Circle>) {
    let mut safe_to_draw = false;
    let mut circle = Circle::new();
    for _ in 0..CREATE_CIRCLE_ATTEMPTS {
        if !has_collision(&circle, circles) {
            safe_to_draw = true;
            break;
        }
        circle = Circle::new();
    }

    if !safe_to_draw {
        return;
    }

    for _ in 0..MAX_RADIUS {
        circle.increase();
        if has_collision(&circle, circles) {
            circle.decrease();
            break;
        }
    }

    circles.push(circle);
}

fn has_collision(circle: &Circle, circles: &Vec<Circle>) -> bool {
    for c in circles {
        let a = c.r.get() + circle.r.get();
        let x = circle.x - c.x;
        let y = circle.y - c.y;

        if a * a >= (x * x + y * y) {
            return true;
        }
    }

    if (circle.x.abs() + circle.r.get()) >= WIDTH as f32 || circle.x.abs() - circle.r.get() <= 0.0 {
        return true;
    }

    if (circle.y.abs() + circle.r.get()) >= HEIGHT as f32 || circle.y.abs() - circle.r.get() <= 0.0
    {
        return true;
    }

    return false;
}
