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

extern crate nannou;

use nannou::prelude::*;
use nannou::app::Draw;
use nannou::geom::rect::Rect;
use nannou::rand::random;
use nannou::math::map_range;
use std::ops::Range;
use nannou::color::Rgb;

fn main() {
    nannou::app(model, event, view).run();
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
        let mut fill = Rgb::new(1.0, 1.0, 1.0);
        if self.previous == 0 && self.state == 1 {
            fill = Rgb::new(0.0, 0.0, 1.0);
        } else if self.previous == 1 && self.state == 0 {
            fill = Rgb::new(1.0, 0.0, 0.0);
        }
        draw.rect()
            .x_y(x, y)
            .w_h(self.w, self.w)
            .rgb(fill.red, fill.green, fill.blue);
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
    fn new(rect: Rect<f32>) -> Self {
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
                            (x as i32 + (i as i32 - 1) + self.columns as i32) % self.columns as i32;
                        let j_idx = (y as i32 + (j as i32 - 1) + self.rows as i32) % self.rows as i32;
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
    fn display(&self, draw: &Draw, rect: &Rect<f32>) {
        for i in 0..self.columns {
            for j in 0..self.rows {
                let x = (i * self.w) as f32 - rect.right() as f32;
                let y = (j * self.w) as f32 - rect.top() as f32;
                self.board[i][j].display(&draw, x, y);
            }
        }
    }
}

struct Model {
    window: WindowId,
    gol: Gol,
}

fn model(app: &App) -> Model {
    let rect = Rect::from_wh(Vector2::new(640.0 * 2.0, 360.0 * 2.0));
    let window = app.new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .build()
        .unwrap();

    let gol = Gol::new(rect);
    Model { window, gol }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            match event {
                // KEY EVENTS
                KeyPressed(_key) => {}

                // MOUSE EVENTS
                MousePressed(_button) => {
                    // Reset board when mouse is pressed
                    m.gol.init();
                }

                _other => (),
            }
        }
        // update gets called just before view every frame
        Event::Update(_dt) => {
            m.gol.generate();
        }
        _ => (),
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("noc_7_3_game_of_life_oop");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    m.gol.display(&draw, &app.window.rect());

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
