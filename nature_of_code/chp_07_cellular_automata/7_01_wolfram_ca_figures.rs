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
    scl: i32,
    cells_range: Range<usize>,
}

impl Ca {
    fn new(r: Vec<i32>, rect: Rect) -> Self {
        let rule_set = r;
        let scl = 20;
        let cells = vec![0; (rect.w() as i32 / scl) as usize];
        let cells_range = 1..cells.len() - 1;
        let generation = 0;

        let mut ca = Ca {
            scl,
            rule_set,
            cells,
            generation,
            cells_range,
        };
        ca.restart();
        ca
    }

    // Set the rules of the CA
    fn _set_rules(&mut self, r: Vec<i32>) {
        self.rule_set = r;
    }

    // Make a random rule set
    fn randomize(&mut self) {
        for i in 0..self.rule_set.len() {
            self.rule_set[i] = (random_f32() * 2.0).floor() as i32;
        }
    }

    // Reset generation to 0
    fn restart(&mut self) {
        for i in 0..self.rule_set.len() {
            self.cells[i] = 0;
        }
        let length = self.cells.len();
        self.cells[length / 2] = 1; // We arbitrarily start with just the middle cell having a state of "1"
        self.generation = 0;
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
    fn display(&self, draw: &DrawHolder, rect: &Rect) {
        for i in 0..self.cells.len() {
            let mut fill = 1.0;
            if self.cells[i] == 1 {
                fill = 0.0;
            }
            draw.rect()
                .x_y(
                    ((self.scl / 2) + i as i32 * self.scl) as f32 - rect.right() as f32,
                    rect.top() as f32 - (self.generation * self.scl - (self.scl / 2)) as f32,
                )
                .w_h(self.scl as f32, self.scl as f32)
                .gray(fill)
                .stroke(BLACK);
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
    fn finished(&self, rect: &Rect) -> bool {
        if self.generation > rect.h() as i32 / self.scl {
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
    let rect = Rect::from_w_h(1800.0, 600.0);
    app.new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .view(view)
        .build();

    let rule_set = vec![0, 1, 1, 1, 1, 0, 1, 1];
    let ca = Ca::new(rule_set, rect);
    Model { ca }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    if m.ca.finished(&app.window_rect()) == false {
        m.ca.generate();
    } else {
        m.ca.randomize();
        m.ca.restart();
    }
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    m.ca.display(&draw, &app.window_rect());



}
