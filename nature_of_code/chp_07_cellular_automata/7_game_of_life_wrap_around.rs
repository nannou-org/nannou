// Daniel Shiffman
// http://natureofcode.com

// Daniel Shiffman, Nature of Code

// A basic implementation of John Conway's Game of Life CA
// how could this be improved to use object oriented programming?
// think of it as similar to our particle system, with a "cell" class
// to describe each individual cell and a "cellular automata" class
// to describe a collection of cells

// Cells wrap around

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Gol {
    w: usize,
    columns: usize,
    rows: usize,
    board: Vec<Vec<i32>>,
}

impl Gol {
    fn new(rect: Rect) -> Self {
        let w = 8;
        let columns = rect.w() as usize / w;
        let rows = rect.h() as usize / w;
        let board = vec![vec![0; rows]; columns];

        let mut gol = Gol {
            w,
            columns,
            rows,
            board,
        };

        gol.init();

        gol
    }

    fn init(&mut self) {
        for i in 0..self.columns {
            for j in 0..self.rows {
                self.board[i][j] = if random::<bool>() { 1 } else { 0 };
            }
        }
    }

    // The process of creating the new generation
    fn generate(&mut self) {
        let mut next = vec![vec![0; self.rows]; self.columns];

        // Loop through every spot in our 2D array and check spots neighbors
        for x in 0..self.columns {
            for y in 0..self.rows {
                // Add up all the states in a 3x3 surrounding grid
                let mut neighbors = 0;
                for i in 0..3i32 {
                    for j in 0..3i32 {
                        let cols = self.columns as i32;
                        let rows = self.rows as i32;
                        let board_x = (x as i32 + (i - 1) + cols) % cols;
                        let board_y = (y as i32 + (j - 1) + rows) % rows;
                        neighbors += self.board[board_x as usize][board_y as usize];
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
    fn display(&self, draw: &Draw, rect: &Rect) {
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
    let rect = Rect::from_w_h(400.0, 400.0);
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
