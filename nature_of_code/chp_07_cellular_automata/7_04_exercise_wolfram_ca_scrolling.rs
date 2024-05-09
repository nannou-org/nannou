// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 7-4: Wolfram Cellular Automata

// Simple demonstration of a Wolfram 1-dimensional cellular automata
// with the system scrolling by
// Also implements wrap around

use nannou::prelude::*;
use std::ops::Range;

const RULE: i32 = 5;

fn main() {
    nannou::app(model).update(update).run();
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
    fn new(r: Vec<i32>, rect: Rect) -> Self {
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
    fn display(&self, draw: &DrawHolder, rect: &Rect) {
        let offset = self.generation % self.rows as i32;
        for col in 0..self.columns {
            for row in 0..self.rows {
                let mut y = row as i32 - offset;
                if y <= rect.top() as i32 {
                    y = self.rows as i32 + y;
                }
                // Only draw if cell state is 1
                let mut fill = 1.0;
                if self.matrix[col][row] == 1 {
                    fill = 0.0;
                }
                let x =
                    ((self.w as i32 / 2) + col as i32 * self.w as i32) as f32 - rect.right() as f32;
                let y = rect.top() - (self.w / 2) as f32 - ((y - 1) * self.w as i32) as f32;
                draw.rect()
                    .x_y(x, y)
                    .w_h(self.w as f32, self.w as f32)
                    .gray(fill);
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
    fn _finished(&self, rect: &Rect) -> bool {
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
    let rect = Rect::from_w_h(640.0, 800.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .view(view)
        .build();

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

fn update(_app: &App, m: &mut Model, _update: Update) {
    m.ca.generate();
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    draw.background().color(WHITE);

    m.ca.display(&draw, &app.window_rect());



}
