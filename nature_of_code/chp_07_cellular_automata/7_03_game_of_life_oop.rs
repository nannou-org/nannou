// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 7-3: Conways Game of Life

// A basic implementation of John Conway's Game of Life CA
// Each cell is now an object!

use nannou::prelude::*;
use std::ops::Range;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone)]
struct Cell {
    w: f32,
    state: i32,
    previous: i32,
}

impl Cell {
    fn new(w: f32) -> Self {
        let w = w;
        let state = if random::<bool>() { 1 } else { 0 };
        let previous = state;
        Cell { w, state, previous }
    }

    fn save_previous(&mut self) {
        self.previous = self.state;
    }

    fn new_state(&mut self, s: i32) {
        self.state = s;
    }

    fn display(&self, draw: &Draw, x: f32, y: f32) {
        let fill = if self.previous == 0 && self.state == 1 {
            Color::srgb(0.0, 0.0, 1.0)
        } else if self.state == 1 {
            Color::gray(0.0)
        } else if self.previous == 1 && self.state == 0 {
            Color::srgb(1.0, 0.0, 0.0)
        } else {
            Color::gray(1.0)
        };
        draw.rect()
            .x_y(x, y)
            .w_h(self.w, self.w)
            .color(fill)
            .stroke(BLACK);
    }
}

struct Gol {
    w: usize,
    columns: usize,
    rows: usize,
    col_range: Range<usize>,
    row_range: Range<usize>,
    board: Vec<Vec<Cell>>,
}

impl Gol {
    fn new(rect: geom::Rect) -> Self {
        let w = 8;
        let columns = rect.w() as usize / w;
        let rows = rect.h() as usize / w;
        let col_range = 1..columns;
        let row_range = 0..rows;
        //let mut board = vec![vec![Cell::new(w as f32); rows]; columns];
        let board = (0..columns)
            .map(|_| (0..rows).map(|_| Cell::new(w as f32)).collect())
            .collect();

        let mut gol = Gol {
            w,
            columns,
            rows,
            col_range,
            row_range,
            board,
        };
        gol.init();
        gol
    }

    fn init(&mut self) {
        self.board = (0..self.columns)
            .map(|_| (0..self.rows).map(|_| Cell::new(self.w as f32)).collect())
            .collect();

        //self.board = vec![vec![Cell::new(self.w as f32); self.rows]; self.columns];
    }

    // The process of creating the new generation
    fn generate(&mut self) {
        for i in 0..self.columns {
            for j in 0..self.rows {
                self.board[i][j].save_previous();
            }
        }

        // Loop through every spot in our 2D array and check spots neighbors
        for x in self.col_range.clone() {
            for y in self.row_range.clone() {
                // Add up all the states in a 3x3 surrounding grid
                let mut neighbors = 0;
                for i in 0..3 {
                    for j in 0..3 {
                        let i_idx =
                            (x as i32 + (i - 1) + self.columns as i32) % self.columns as i32;
                        let j_idx =
                            (y as i32 + (j - 1) + self.rows as i32) % self.rows as i32;
                        neighbors += self.board[i_idx as usize][j_idx as usize].previous;
                    }
                }

                // A little trick to subtract the current call's state since
                // we added it in the above loop
                neighbors -= self.board[x][y].previous;

                // Rules of Life
                if self.board[x][y].state == 1 && neighbors < 2 {
                    self.board[x][y].new_state(0); // Loneliness
                } else if self.board[x][y].state == 1 && neighbors > 3 {
                    self.board[x][y].new_state(0); // Over Population
                } else if self.board[x][y].state == 0 && neighbors == 3 {
                    self.board[x][y].new_state(1); // Reproduction
                }
            }
        }
    }

    // This is the easy part, just draw the cells fill white if 1, black if 0
    fn display(&self, draw: &Draw, rect: &geom::Rect) {
        for i in 0..self.columns {
            for j in 0..self.rows {
                let x = (i * self.w) as f32 - rect.right();
                let y = (j * self.w) as f32 - rect.top();
                let offset = self.w as f32 / 2.0;
                self.board[i][j].display(draw, x + offset, y + offset);
            }
        }
    }
}

struct Model {
    gol: Gol,
}

fn model(app: &App) -> Model {
    let rect = geom::Rect::from_w_h(640.0, 360.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .mouse_pressed(mouse_pressed)
        .view(view)
        .build();

    let gol = Gol::new(rect);
    Model { gol }
}

fn mouse_pressed(_app: &App, m: &mut Model, _button: MouseButton) {
    // Reset board when mouse is pressed
    m.gol.init();
}

fn update(_app: &App, m: &mut Model) {
    m.gol.generate();
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.gol.display(&draw, &app.window_rect());
}
