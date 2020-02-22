// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Outline for game of life
// This is just a grid of hexagons right now

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone)]
struct Cell {
    x: f32,
    y: f32,
    w: f32,
    state: i32,
}

impl Cell {
    fn new(x: f32, y: f32, w: f32) -> Self {
        let state = if random::<bool>() { 1 } else { 0 };
        Cell { x, y, w, state }
    }

    fn display(&self, draw: &app::Draw, rect: &Rect) {
        let fill = if self.state == 1 {
            gray(0.0)
        } else {
            gray(1.0)
        };

        let n_points = 6;
        let radius = self.w;
        let points = (0..n_points).map(|i| {
            let fract = i as f32 / n_points as f32;
            let phase = fract;
            let x = radius * (TAU * phase).cos();
            let y = radius * (TAU * phase).sin();
            pt2(x, y)
        });
        let x_off = rect.left() + self.w / 2.0;
        let y_off = rect.top() - self.w / 2.0;
        draw.polygon()
            .x_y(x_off + self.x, y_off - self.y)
            .color(fill)
            .stroke(BLACK)
            .points(points);
    }
}

struct Gol {
    w: f32,
    h: f32,
    columns: usize,
    rows: usize,
    board: Vec<Vec<Cell>>,
}

impl Gol {
    fn new(rect: Rect) -> Self {
        let w = 20.0;
        let h = deg_to_rad(60.0).sin() * w;
        let columns = (rect.w() / (w * 3.0)) as usize;
        let rows = (rect.h() / h) as usize;
        let board = vec![vec![Cell::new(0.0, 0.0, w); rows]; columns];

        let mut gol = Gol {
            w,
            h,
            columns,
            rows,
            board,
        };
        gol.init();
        gol
    }

    fn init(&mut self) {
        self.board = (0..self.columns)
            .map(|i| {
                (0..self.rows)
                    .map(|j| {
                        if j % 2 == 0 {
                            Cell::new(i as f32 * self.w * 3.0, j as f32 * self.h, self.w)
                        } else {
                            Cell::new(
                                i as f32 * self.w * 3.0 + self.w + self.w / 2.0,
                                j as f32 * self.h,
                                self.w,
                            )
                        }
                    })
                    .collect()
            })
            .collect();
    }

    // This is the easy part, just draw the cells fill white if 1, black if 0
    fn display(&self, draw: &app::Draw, rect: &Rect) {
        for i in 0..self.columns {
            for j in 0..self.rows {
                self.board[i][j].display(&draw, &rect);
            }
        }
    }
}

struct Model {
    gol: Gol,
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(600.0, 600.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .mouse_pressed(mouse_pressed)
        .view(view)
        .build()
        .unwrap();

    let gol = Gol::new(rect);
    Model { gol }
}

fn mouse_pressed(_app: &App, m: &mut Model, _button: MouseButton) {
    // Reset board when mouse is pressed
    m.gol.init();
}

fn update(_app: &App, _m: &mut Model, _update: Update) {}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.gol.display(&draw, &app.window_rect());

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
