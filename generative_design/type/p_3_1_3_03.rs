// P_3_1_3_03
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
 * drawing the letters frequency with lines and ellipses
 *
 * MOUSE
 * position x          : random angle
 * position y          : line length, size of ellipses, tracking
 *
 * KEYS
 * 1                   : toggle alpha mode
 * 2                   : toggle drawing of lines
 * 3                   : toggle drawing of ellipses
 * 4                   : toggle drawing of text
 * s                   : save png
 */
use nannou::prelude::*;
use nannou::rand::rngs::StdRng;
use nannou::rand::{Rng, SeedableRng};

fn main() {
    nannou::app(model).run();
}

struct Model {
    joined_text: String,
    alphabet: String,
    counters: Vec<u32>,
    draw_alpha: bool,
    draw_lines: bool,
    draw_ellipses: bool,
    draw_text: bool,
    act_random_seed: u64,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1200, 800)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_released(key_released)
        .build();

    let text_path = app.assets_path().join("text").join("faust_kurz.txt");
    let joined_text = std::fs::read_to_string(text_path).unwrap().parse().unwrap();
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÜß,.;:!? ".to_string();
    let counters = vec![0; alphabet.len()];

    let mut model = Model {
        joined_text,
        alphabet,
        counters,
        draw_alpha: true,
        draw_lines: true,
        draw_ellipses: true,
        draw_text: false,
        act_random_seed: 47,
    };

    count_characters(&mut model);
    model
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();
    draw.background().color(WHITE);

    let mut pos_x = win.left() + 80.0;
    let mut pos_y = win.top() - 300.0;

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    // go through all characters in the text to draw them
    for c in model.joined_text.chars() {
        // again, find the index of the current letter in the character set
        let upper_case_char = c.to_uppercase().next().unwrap();
        let index = model.alphabet.chars().position(|c| c == upper_case_char);
        if index.is_none() {
            continue;
        }

        // calculate parameters
        let mut char_alpha = 1.0;
        if model.draw_alpha {
            char_alpha = model.counters[index.unwrap()] as f32 / 100.0;
        }

        let my = clamp(
            map_range(
                app.mouse().x,
                win.top() - 50.0,
                win.bottom() + 50.0,
                0.0,
                1.0,
            ),
            0.0,
            1.0,
        );
        let char_size = model.counters[index.unwrap()] as f32 * my * 3.0;

        let mx = clamp(
            map_range(
                app.mouse().x,
                win.left() + 50.0,
                win.right() - 50.0,
                0.0,
                1.0,
            ),
            0.0,
            1.0,
        );
        let line_length = char_size;
        let line_angle = rng.random_range(-PI..PI) * mx * (PI / 2.0);
        let new_pos_x = line_length * line_angle.cos();
        let new_pos_y = line_length * line_angle.sin();

        // draw elements
        let draw = draw.x_y(pos_x, pos_y);
        if model.draw_lines {
            draw.line()
                .start(pt2(0.0, 0.0))
                .end(pt2(new_pos_x, new_pos_y))
                .hsva(0.75, 0.73, 0.51, char_alpha);
        }
        if model.draw_ellipses {
            draw.ellipse()
                .x_y(0.0, 0.0)
                .radius(char_size / 20.0)
                .hsva(0.14, 1.0, 0.71, char_alpha);
        }
        if model.draw_text {
            let character = &c.to_string();
            let text = text(character).font_size(18).build(win);
            draw.path()
                .fill()
                .x_y(new_pos_x, new_pos_y)
                .srgba(0.0, 0.0, 0.0, char_alpha)
                .events(text.path_events());
        }

        pos_x += 9.0;
        if pos_x >= win.right() - 200.0 && upper_case_char == ' ' {
            let tracking = 27.0;
            pos_y -= tracking * my + 30.0;
            pos_x = win.left() + 80.0;
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

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        KeyCode::Digit1 => {
            model.draw_alpha = !model.draw_alpha;
        }
        KeyCode::Digit2 => {
            model.draw_lines = !model.draw_lines;
        }
        KeyCode::Digit3 => {
            model.draw_ellipses = !model.draw_ellipses;
        }
        KeyCode::Digit4 => {
            model.draw_text = !model.draw_text;
        }
        _other_key => {}
    }
}
