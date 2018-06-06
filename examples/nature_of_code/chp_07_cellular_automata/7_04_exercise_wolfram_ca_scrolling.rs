// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 7-4: Wolfram Cellular Automata

// Simple demonstration of a Wolfram 1-dimensional cellular automata
// with the system scrolling by
// Also implements wrap around

extern crate nannou;

use nannou::prelude::*;
use std::ops::Range;

const RULE: i32 = 2;

fn main() {
    nannou::app(model, event, view).run();
}

// A Type to manage the CA
struct Ca {
    generation: i32,    // How many generations?
    rule_set: Vec<i32>, // An array to store the ruleset, for example {0,1,1,0,1,1,0,1}
    w: usize,
    matrix: Vec<Vec<i32>>,
    columns: usize,
    rows: usize,
    col_range: Range<usize>,
}

impl Ca {
    fn new(r: Vec<i32>, rect: Rect<f32>) -> Self {
        let rule_set = r;
        let generation = 0;
        let w = 4;
        let columns = rect.w() as usize / w;
        let rows = rect.h() as usize / w;
        let col_range = 1..columns - 1;
        let matrix = vec![vec![0; rows]; columns];

        let mut ca = Ca {
            rule_set,
            generation,
            w,
            columns,
            rows,
            col_range,
            matrix,
        };
        ca.restart();
        ca
    }

    // Make a random rule set
    fn _randomize(&mut self) {
        self.rule_set = (0..self.rule_set.len())
            .map(|_| random_range(0i32, 2))
            .collect();
    }

    // Reset generation to 0
    fn restart(&mut self) {
        self.matrix = vec![vec![0; self.rows]; self.columns];
        self.matrix[self.columns / 2][0] = 1; // We arbitrarily start with just the middle cell having a state of "1"
        self.generation = 0;
    }

    // The process of creating the new generation
    fn generate(&mut self) {
        // For every spot, determine new state by examing current state, and neighbor states
        // Ignore edges that only have one neighor
        for i in self.col_range.clone() {
            let left = self.matrix[(i + self.columns - 1) % self.columns]
                [(self.generation % self.rows as i32) as usize]; // Left neighbor state
            let me = self.matrix[i][(self.generation % self.rows as i32) as usize]; // Current state
            let right =
                self.matrix[(i + 1) % self.columns][(self.generation % self.rows as i32) as usize]; // Right beighbor state
            self.matrix[i][((self.generation + 1) % self.rows as i32) as usize] =
                self.rules(left, me, right); // Compute next generation state based on ruleset
        }
        self.generation += 1;
    }

    // This is the easy part, just draw the cells fill white if 1, black if 0
    fn display(&self, draw: &app::Draw, rect: &Rect<f32>) {
        let offset = (self.generation % self.rows as i32) as usize;
        for i in 0..self.columns {
            for j in 0..self.rows {
                let mut y = j - offset;
                // if y <= 0 {
                if y >= rect.top() as usize {
                    y = self.rows + y;
                }
                // Only draw if cell state is 1
                let mut fill = 1.0;
                if self.matrix[i][j] == 1 {
                    fill = 0.0;
                }
                draw.rect()
                    .x_y(
                        ((self.w as i32 / 2) + i as i32 * self.w as i32) as f32
                            - rect.right() as f32,
                        rect.top() - (self.w / 2) as f32 - ((y - 1) * self.w) as f32,
                        //                        rect.top() as f32 - ((y - 1) * self.w + (self.w / 2)) as f32,
                    )
                    .w_h(self.w as f32, self.w as f32)
                    .rgb(fill, fill, fill);
            }
        }
    }

    // Implementing the Wolfram rules
    // Could be improved and made more concise, but here we can explicitly see what is going on for each case
    fn rules(&self, a: i32, b: i32, c: i32) -> i32 {
        if a == 1 && b == 1 && c == 1 {
            return self.rule_set[0];
        }
        if a == 1 && b == 1 && c == 0 {
            return self.rule_set[1];
        }
        if a == 1 && b == 0 && c == 1 {
            return self.rule_set[2];
        }
        if a == 1 && b == 0 && c == 0 {
            return self.rule_set[3];
        }
        if a == 0 && b == 1 && c == 1 {
            return self.rule_set[4];
        }
        if a == 0 && b == 1 && c == 0 {
            return self.rule_set[5];
        }
        if a == 0 && b == 0 && c == 1 {
            return self.rule_set[6];
        }
        if a == 0 && b == 0 && c == 0 {
            return self.rule_set[7];
        }
        0
    }

    // The CA is done if it reaches the bottom of the screen
    fn _finished(&self, rect: &Rect<f32>) -> bool {
        if self.generation > rect.h() as i32 / self.w as i32 {
            true
        } else {
            false
        }
    }
}

struct Model {
    ca: Ca,
}

fn model(app: &App) -> Model {
    let rect = Rect::from_wh(Vector2::new(640.0, 800.0));
    let _window = app.new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .build()
        .unwrap();

    let rule_set = match RULE {
        1 => vec![0, 1, 1, 1, 1, 0, 1, 1], // Rule 222
        2 => vec![0, 1, 1, 1, 1, 1, 0, 1], // Rule 190
        3 => vec![0, 1, 1, 1, 1, 0, 0, 0], // Rule 30
        4 => vec![0, 1, 1, 1, 0, 1, 1, 0], // Rule 110
        5 => vec![0, 1, 0, 1, 1, 0, 1, 0], // Rule 90
        _ => vec![0, 0, 0, 0, 0, 0, 0, 0],
    };

    let ca = Ca::new(rule_set, rect);
    Model { ca }
}

fn event(_app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        m.ca.generate();
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();

    m.ca.display(&draw, &app.window_rect());

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
