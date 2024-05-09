// P_3_1_3_02
//
// Generative Gestaltung – Creative Coding im Web
// ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
// Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
// with contributions by Joey Lee and Niels Poldervaart
// Copyright 2018
//
// http://www.generative-gestaltung.de
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/**
 * analysing and sorting the letters of a text
 * connecting subsequent letters with lines
 *
 * MOUSE
 * position x          : interpolate between normal text and sorted position
 *
 * KEYS
 * 1                   : toggle lines on/off
 * 2                   : toggle text on/off
 * 3                   : switch all letters off
 * 4                   : switch all letters on
 * a-z                 : switch letter on/off
 * CONTROL             : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    joined_text: String,
    alphabet: String,
    counters: Vec<u32>,
    draw_letters: Vec<bool>,
    draw_lines: bool,
    draw_text: bool,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1200, 800)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    let text_path = app
        .assets_path()
        .unwrap()
        .join("text")
        .join("faust_kurz.txt");
    let joined_text = std::fs::read_to_string(text_path).unwrap().parse().unwrap();
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÜß,.;:!? ".to_string();
    let counters = vec![0; alphabet.len()];
    let draw_letters = vec![true; alphabet.len()];

    let mut model = Model {
        joined_text,
        alphabet,
        counters,
        draw_letters,
        draw_lines: false,
        draw_text: true,
    };

    count_characters(&mut model);
    model
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();
    draw.background().color(WHITE);

    let mut pos_x = win.left() + 20.0;
    let mut pos_y = win.top() - 200.0;
    let mut old_x = 0.0;
    let mut old_y = 0.0;

    // go through all characters in the text to draw them
    for c in model.joined_text.chars() {
        // again, find the index of the current letter in the character set
        let upper_case_char = c.to_uppercase().next().unwrap();
        let index = model.alphabet.chars().position(|c| c == upper_case_char);
        if index.is_none() {
            continue;
        }

        let sort_y = win.top() - (index.unwrap() * 20 + 40) as f32;
        let m = clamp(
            map_range(app.mouse().x, win.left() + 50.0, win.right() - 50.0, 0.0, 1.0),
            0.0,
            1.0,
        );
        let inter_y = nannou::geom::range::Range::new(pos_y, sort_y).lerp(m);

        if model.draw_letters[index.unwrap()] {
            if model.draw_lines {
                if old_x != 0.0 && old_y != 0.0 {
                    draw.line()
                        .start(pt2(old_x, old_y))
                        .end(pt2(pos_x, inter_y))
                        .weight(1.5)
                        .rgba(0.7, 0.6, 0.0, 0.6);
                }
                old_x = pos_x;
                old_y = inter_y;
            }
            if model.draw_text {
                let character = &c.to_string();
                let text = text(character).font_size(18).build(win);
                draw.path()
                    .fill()
                    .x_y(pos_x, inter_y)
                    .rgb(0.34, 0.13, 0.5)
                    .events(text.path_events());
            }
        } else {
            old_x = 0.0;
            old_y = 0.0;
        }

        pos_x += 9.0;
        if pos_x >= win.right() - 200.0 && upper_case_char == ' ' {
            pos_y -= 40.0;
            pos_x = win.left() + 20.0;
        }
    }



}

fn count_characters(model: &mut Model) {
    for c in model.joined_text.chars() {
        // get one character from the text and turn it to uppercase
        let upper_case_char = c.to_uppercase().next().unwrap();
        let index = model.alphabet.chars().position(|c| c == upper_case_char);
        if index.is_some() {
            // increase the respective counter
            model.counters[index.unwrap()] += 1;
        }
    }
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::LControl | KeyCode::KeyRControl => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        KeyCode::Digit1 => {
            model.draw_lines = !model.draw_lines;
        }
        KeyCode::Digit2 => {
            model.draw_text = !model.draw_text;
        }
        KeyCode::Digit3 => {
            for i in 0..model.alphabet.len() {
                model.draw_letters[i] = false;
            }
        }
        KeyCode::Digit4 => {
            for i in 0..model.alphabet.len() {
                model.draw_letters[i] = true;
            }
        }
        _other_key => {}
    }
}
