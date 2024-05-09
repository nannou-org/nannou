// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 7-2: Conways Game of Life

// A basic implementation of John Conway's Game of Life CA
// how could this be improved to use object oriented programming?
// think of it as similar to our particle system, with a "cell" class
// to describe each individual cell and a "cellular automata" class
// to describe a collection of cells

use nannou::prelude::*;
use std::ops::Range;

fn main() {
    nannou::app(model).update(update).run();
}

struct Gol {
    w: usize,
    columns: usize,
    rows: usize,
    col_range: Range<usize>,
    row_range: Range<usize>,
    board: Vec<Vec<i32>>,
}

impl Gol {
    fn new(rect: Rect) -> Self {
        let w = 8;
        let columns = rect.w() as usize / w;
        let rows = rect.h() as usize / w;
        let col_range = 1..columns - 1;
        let row_range = 1..rows - 1;
        let board = vec![vec![0; rows]; columns];
        /*
        // This is how to initilase a 2d vector
        let board = (0..rows)
            .map(|_| {
                (0..columns)
                    .map(|_| if random::<bool>() {1.0} else {0.0})
                    .collect()
            })
            .collect();
        */

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
        for i in self.col_range.clone() {
            for j in self.row_range.clone() {
                self.board[i][j] = if random::<bool>() { 1 } else { 0 };
            }
        }
    }

    // The process of creating the new generation
    fn generate(&mut self) {
        let mut next = vec![vec![0; self.rows]; self.columns];

        // Loop through every spot in our 2D array and check spots neighbors
        for x in self.col_range.clone() {
            for y in self.row_range.clone() {
                // Add up all the states in a 3x3 surrounding grid
                let mut neighbors = 0;
                for i in 0..3 {
                    for j in 0..3 {
                        neighbors += self.board[(x as i32 + (i as i32 - 1)) as usize]
                            [(y as i32 + (j as i32 - 1)) as usize];
                    }
                }

                // A little trick to subtract the current call's state since
                // we added it in the above loop
                neighbors -= self.board[x][y];

                // Rules of Life
                if self.board[x][y] == 1 && neighbors < 2 {
                    next[x][y] = 0; // Loneliness
                } else if self.board[x][y] == 1 && neighbors > 3 {
                    next[x][y] = 0; // Over Population
                } else if self.board[x][y] == 0 && neighbors == 3 {
                    next[x][y] = 1; // Reproduction
                } else {
                    next[x][y] = self.board[x][y]; // Stasis
                }
            }
        }
        // Next is now our board
        self.board = next;
    }

    // This is the easy part, just draw the cells fill white if 1, black if 0
    fn display(&self, draw: &DrawHolder, rect: &Rect) {
        for i in 0..self.columns {
            for j in 0..self.rows {
                let mut fill = 1.0;
                if self.board[i][j] == 1 {
                    fill = 0.0;
                }
                let offset = self.w as f32 / 2.0;
                draw.rect()
                    .x_y(
                        offset + (i * self.w) as f32 - rect.right() as f32,
                        offset + (j * self.w) as f32 - rect.top() as f32,
                    )
                    .w_h(self.w as f32, self.w as f32)
                    .gray(fill)
                    .stroke(BLACK);
            }
        }
    }
}

struct Model {
    gol: Gol,
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(640.0, 360.0);
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

fn update(_app: &App, m: &mut Model, _update: Update) {
    m.gol.generate();
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.gol.display(&draw, &app.window_rect());



}
