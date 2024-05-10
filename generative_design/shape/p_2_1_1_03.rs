// P_2_1_1_03
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
 * changing number, color and strokeweight on diagonals in a grid
 *
 * MOUSE
 * position x          : diagonal strokeweight
 * position y          : number diagonals
 * left click          : new random layout
 *
 * KEYS
 * s                   : save png
 * 1                   : color left diagonal
 * 2                   : color right diagonal
 * 3                   : switch transparency left diagonal on/off
 * 4                   : switch transparency right diagonal on/off
 * 0                   : default
 */
use nannou::prelude::*;
use nannou::rand::rngs::StdRng;
use nannou::rand::{Rng, SeedableRng};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    tile_count: u32,
    act_random_seed: u64,
    color_left: Hsva,
    color_right: Hsva,
    transparent_left: bool,
    transparent_right: bool,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_released(key_released)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    Model {
        tile_count: 1,
        act_random_seed: 0,
        color_left: Color::hsv(0.88, 1.0, 0.77).into(),
        color_right: Color::hsv(0.0, 0.0, 0.0).into(),
        transparent_left: false,
        transparent_right: false,
    }
}

fn update(app: &App, model: &mut Model) {
    model.tile_count = (app.mouse().y + app.window_rect().top()) as u32 / 15;
}

fn view(app: &App, model: &Model) {
    // Prepare to draw.
    let draw = app.draw();
    draw.background().color(WHITE);
    let win = app.window_rect();

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    for grid_y in 0..model.tile_count {
        for grid_x in 0..model.tile_count {
            let tile_w = win.w() / model.tile_count as f32;
            let tile_h = win.h() / model.tile_count as f32;
            let pos_x = win.left() + tile_w * grid_x as f32;
            let pos_y = (win.top() - tile_h) - tile_h * grid_y as f32;
            let mx = clamp(win.right() + app.mouse().x, 0.0, win.w());

            let toggle = rng.gen::<bool>();

            if toggle == false {
                let (h, s, v) = model.color_left.into_components();
                let a = calculate_alpha_left(grid_y, model.transparent_left);

                draw.line()
                    .hsva(h.into(), s, v, a)
                    .caps_round()
                    .weight(mx / 15.0)
                    .points(
                        pt2(pos_x, pos_y),
                        pt2(
                            pos_x + (win.w() / model.tile_count as f32) / 2.0,
                            pos_y + win.h() / model.tile_count as f32,
                        ),
                    );
                draw.line()
                    .hsva(h.into(), s, v, a)
                    .caps_round()
                    .weight(mx / 15.0)
                    .points(
                        pt2(pos_x + (win.w() / model.tile_count as f32) / 2.0, pos_y),
                        pt2(
                            pos_x + (win.w() / model.tile_count as f32),
                            pos_y + win.h() / model.tile_count as f32,
                        ),
                    );
            }
            if toggle == true {
                let (h, s, v) = model.color_right.into_components();
                let a = calculate_alpha_right(grid_y, model.transparent_right);

                draw.line()
                    .hsva(h.into(), s, v, a)
                    .caps_round()
                    .weight(mx / 15.0)
                    .points(
                        pt2(pos_x, pos_y + win.w() / model.tile_count as f32),
                        pt2(pos_x + (win.h() / model.tile_count as f32) / 2.0, pos_y),
                    );
                draw.line()
                    .hsva(h.into(), s, v, a)
                    .caps_round()
                    .weight(mx / 15.0)
                    .points(
                        pt2(
                            pos_x + (win.h() / model.tile_count as f32) / 2.0,
                            pos_y + win.w() / model.tile_count as f32,
                        ),
                        pt2(pos_x + win.h() / model.tile_count as f32, pos_y),
                    );
            }
        }
    }

    // Write to the window frame.

}

fn calculate_alpha_left(grid_y: u32, transparent_left: bool) -> f32 {
    let max_alpha = 100.0;
    let alpha_left = if transparent_left {
        let a = grid_y as f32 * 10.0;
        match a {
            _ if a > max_alpha => 1.0,
            _ if a < 0.0 => 0.0,
            _ => a / max_alpha,
        }
    } else {
        1.0
    };

    alpha_left
}

fn calculate_alpha_right(grid_y: u32, transparent_right: bool) -> f32 {
    let max_alpha = 100.0;
    let alpha_right = if transparent_right {
        let a = max_alpha - (grid_y as f32 * 10.0);
        match a {
            _ if a > max_alpha => 1.0,
            _ if a < 0.0 => 0.0,
            _ => a / max_alpha,
        }
    } else {
        1.0
    };

    alpha_right
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}

fn key_pressed(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}

fn key_released(_app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::Digit1 => {
            if model.color_left.eq(&Color::hsv(0.75, 0.73, 0.51)) {
                model.color_left = Color::hsv(0.89, 1.0, 0.77).into();
            } else {
                model.color_left = Color::hsv(0.75, 0.73, 0.51).into();
            }
        }
        KeyCode::Digit2 => {
            if model.color_right.eq(&Color::hsv(0.0, 0.0, 0.0)) {
                model.color_right = Color::hsv(0.53, 1.0, 0.64).into()
            } else {
                model.color_right = Color::hsv(0.0, 0.0, 0.0).into();
            }
        }
        KeyCode::Digit3 => {
            model.transparent_left = !model.transparent_left;
        }
        KeyCode::Digit4 => {
            model.transparent_right = !model.transparent_right;
        }
        KeyCode::Digit0 => {
            model.transparent_left = false;
            model.transparent_right = false;
            model.color_left = Color::hsv(0.89, 1.0, 0.77).into();
            model.color_right = Color::hsv(0.0, 0.0, 0.0).into();
        }
        _other_key => {}
    }
}
