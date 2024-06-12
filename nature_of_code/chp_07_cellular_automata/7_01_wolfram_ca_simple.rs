// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 7-1: Wolfram Cellular Automata

// Simple demonstration of a Wolfram 1-dimensional cellular automata

use nannou::prelude::*;
use std::ops::Range;

fn main() {
    nannou::app(model).update(update).run();
}

// A Type to manage the CA
struct Ca {
    cells: Vec<i32>,    // An array of 0s and 1s
    generation: i32,    // How many generations?
    rule_set: Vec<i32>, // An array to store the ruleset, for example {0,1,1,0,1,1,0,1}
    w: i32,
    cells_range: Range<usize>,
}

impl Ca {
    fn new(rect: Rect) -> Self {
        let w = 10;
        let rule_set = vec![0, 1, 0, 1, 1, 0, 1, 0];
        let mut cells = vec![0; (rect.w() as i32 / w) as usize];
        let length = cells.len();
        cells[length / 2 as usize] = 1; // We arbitrarily start with just the middle cell having a state of "1"
        let generation = 0;
        let cells_range = 1..cells.len() - 1;
        Ca {
            w,
            rule_set,
            cells,
            generation,
            cells_range,
        }
    }

    // The process of creating the new generation
    fn generate(&mut self) {
        // First we create an empty array for the new values
        let mut next_gen = vec![0; self.cells.len()];

        // For every spot, determine new state by examing current state, and neighbor states
        // Ignore edges that only have one neighor
        for i in self.cells_range.clone() {
            let left = self.cells[i - 1]; // Left neighbor state
            let me = self.cells[i]; // Current state
            let right = self.cells[i + 1]; // Right beighbor state
            next_gen[i] = self.rules(left, me, right); // Compute next generation state based on ruleset
        }
        // The current generation is the new generation
        self.cells = next_gen;
        self.generation += 1;
    }

    // This is the easy part, just draw the cells fill white if 1, black if 0
    fn display(&self, draw: &Draw, rect: &Rect) {
        for i in 0..self.cells.len() {
            let mut fill = 1.0;
            if self.cells[i] == 1 {
                fill = 0.0;
            }
            draw.rect()
                .x_y(
                    ((self.w / 2) + i as i32 * self.w) as f32 - rect.right() as f32,
                    rect.top() as f32 - (self.generation * self.w - (self.w / 2)) as f32,
                )
                .w_h(self.w as f32, self.w as f32)
                .gray(fill);
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
}

struct Model {
    ca: Ca,
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(800.0, 400.0);
    let _window = app
        .new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .view(view)
        .build();

    let ca = Ca::new(rect);
    Model { ca }
}

fn update(app: &App, m: &mut Model) {
    if m.ca.generation < app.window_rect().h() as i32 / m.ca.w {
        m.ca.generate();
    }
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();

    m.ca.display(&draw, &app.window_rect());
}
