// P_3_1_3_04
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
 * 1                   : toggle grey lines on/off
 * 2                   : toggle colored lines on/off
 * 3                   : toggle text on/off
 * 4                   : switch all letters off
 * 5                   : switch all letters on
 * a-z                 : switch letter on/off
 * ctrl                : save png
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
    draw_grey_lines: bool,
    draw_colored_lines: bool,
    draw_text: bool,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1300, 800)
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
        draw_grey_lines: false,
        draw_colored_lines: true,
        draw_text: true,
    };

    count_characters(&mut model);
    model
}

fn view(app: &App, model: &Model) {
    let win = app.window_rect();
    let draw = app.draw().x_y(win.left() + 50.0, 0.0);
    draw.background().color(WHITE);

    let mut pos_x = 0.0;
    let mut pos_y = win.top() - 200.0;
    let mut old_x = 0.0;
    let mut old_y = 0.0;
    let mut sort_positions_x = vec![0.0; model.joined_text.len()];
    let mut old_positions_x = vec![0.0; model.joined_text.len()];
    let mut old_positions_y = vec![0.0; model.joined_text.len()];

    // draw counters
    if app.mouse().x >= win.right() - 50.0 {
        for (i, c) in model.alphabet.chars().enumerate() {
            let character = &c.to_string();
            let size = 10;
            let text1 = text(character).font_size(size).build(win);
            draw.path()
                .fill()
                .x_y(
                    -15.0,
                    win.top() - (i as f32 * 20.0 + 40.0) + (size as f32 / 2.0),
                )
                .color(GREY)
                .events(text1.path_events());

            let digit = &model.counters[i].to_string();
            let text2 = text(digit).font_size(10).build(win);
            draw.path()
                .fill()
                .x_y(
                    -30.0,
                    win.top() - (i as f32 * 20.0 + 40.0) + (size as f32 / 2.0),
                )
                .color(GREY)
                .events(text2.path_events());
        }
    }

    // go through all characters in the text to draw them
    for c in model.joined_text.chars() {
        // again, find the index of the current letter in the character set
        let upper_case_char = c.to_uppercase().next().unwrap();
        let index = model.alphabet.chars().position(|c| c == upper_case_char);
        if index.is_none() {
            continue;
        }

        let m = clamp(
            map_range(app.mouse().x, win.left() + 50.0, win.right() - 50.0, 0.0, 1.0),
            0.0,
            1.0,
        );

        let sort_x = sort_positions_x[index.unwrap()];
        let inter_x = nannou::geom::range::Range::new(pos_x, sort_x).lerp(m);

        let sort_y = win.top() - (index.unwrap() as f32 * 20.0 + 40.0);
        let inter_y = nannou::geom::range::Range::new(pos_y, sort_y).lerp(m);

        if model.draw_letters[index.unwrap()] {
            if model.draw_grey_lines {
                if old_x != 0.0 && old_y != 0.0 {
                    draw.line()
                        .start(pt2(old_x, old_y))
                        .end(pt2(inter_x, inter_y))
                        .hsla(0.0, 0.0, 0.0, 0.1);
                }
                old_x = inter_x;
                old_y = inter_y;
            }

            if model.draw_colored_lines {
                if old_positions_x[index.unwrap()] != 0.0 && old_positions_y[index.unwrap()] != 0.0
                {
                    draw.line()
                        .start(pt2(
                            old_positions_x[index.unwrap()],
                            old_positions_y[index.unwrap()],
                        ))
                        .end(pt2(inter_x, inter_y))
                        .weight(1.5)
                        .hsla(index.unwrap() as f32 * 10.0 / 360.0, 0.8, 0.6, 0.5);
                }
                old_positions_x[index.unwrap()] = inter_x;
                old_positions_y[index.unwrap()] = inter_y;
            }

            if model.draw_text {
                let character = &c.to_string();
                let size = 18;
                let text = text(character).font_size(size).build(win);
                draw.path()
                    .fill()
                    .x_y(inter_x, inter_y + (size as f32 / 2.0))
                    .color(BLACK)
                    .events(text.path_events());
            }
        } else {
            old_x = 0.0;
            old_y = 0.0;
        }

        sort_positions_x[index.unwrap()] += 14.0;
        pos_x += 9.0;
        if pos_x >= win.w() - 200.0 && upper_case_char == ' ' {
            pos_y -= 40.0;
            pos_x = 0.0;
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
            model.draw_grey_lines = !model.draw_grey_lines;
        }
        KeyCode::Digit2 => {
            model.draw_colored_lines = !model.draw_colored_lines;
        }
        KeyCode::Digit3 => {
            model.draw_text = !model.draw_text;
        }
        KeyCode::Digit4 => {
            for i in 0..model.alphabet.len() {
                model.draw_letters[i] = false;
            }
        }
        KeyCode::Digit5 => {
            for i in 0..model.alphabet.len() {
                model.draw_letters[i] = true;
            }
        }
        _other_key => {}
    }
}
