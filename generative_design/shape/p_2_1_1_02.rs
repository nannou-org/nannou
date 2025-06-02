// P_2_1_1_02
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
 * changing strokeweight on diagonals in a grid with colors
 *
 * MOUSE
 * position x          : left diagonal strokeweight
 * position y          : right diagonal strokeweight
 * left click          : new random layout
 *
 * KEYS
 * s                   : save png
 * 1                   : round strokecap
 * 2                   : square strokecap
 * 3                   : project strokecap
 * 4                   : color left diagonal
 * 5                   : color right diagonal
 * 6                   : transparency left diagonal
 * 7                   : transparency right diagonal
 * 0                   : default
 */
use nannou::lyon::tessellation::LineCap;
use nannou::prelude::*;
use nannou::rand::rngs::StdRng;
use nannou::rand::{Rng, SeedableRng};

fn main() {
    nannou::app(model).run();
}

struct Model {
    tile_count: u32,
    act_random_seed: u64,
    act_stroke_cap: LineCap,
    color_left: Srgba,
    color_right: Srgba,
    alpha_left: f32,
    alpha_right: f32,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_released(key_released)
        .key_pressed(key_pressed)
        .build();

    let alpha_left = 1.0;
    let alpha_right = 1.0;

    Model {
        tile_count: 20,
        act_random_seed: 0,
        act_stroke_cap: LineCap::Round,
        color_left: Srgba::new(0.77, 0.0, 0.48, alpha_left),
        color_right: Srgba::new(0.34, 0.137, 0.5, alpha_right),
        alpha_left,
        alpha_right,
    }
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
            let my = clamp(win.top() - app.mouse().x, 0.0, win.h());

            let toggle = rng.random::<bool>();

            if !toggle {
                draw.line()
                    .color(model.color_left)
                    .weight(mx / 10.0)
                    .caps(model.act_stroke_cap)
                    .points(
                        pt2(pos_x, pos_y),
                        pt2(
                            pos_x + win.w() / model.tile_count as f32,
                            pos_y + win.h() / model.tile_count as f32,
                        ),
                    );
            }
            if toggle {
                draw.line()
                    .color(model.color_right)
                    .weight(my / 10.0)
                    .caps(model.act_stroke_cap)
                    .points(
                        pt2(pos_x, pos_y + win.w() / model.tile_count as f32),
                        pt2(pos_x + win.h() / model.tile_count as f32, pos_y),
                    );
            }
        }
    }
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}

fn key_pressed(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window()
            .save_screenshot(app.exe_name().unwrap() + ".png");
    }
}

fn key_released(_app: &App, model: &mut Model, key: KeyCode) {
    let black = Srgba::new(0.0, 0.0, 0.0, 1.0);

    match key {
        KeyCode::Digit1 => {
            model.act_stroke_cap = LineCap::Round;
        }
        KeyCode::Digit2 => {
            model.act_stroke_cap = LineCap::Square;
        }
        KeyCode::Digit3 => {
            model.act_stroke_cap = LineCap::Butt;
        }
        KeyCode::Digit4 => {
            if model.color_left.eq(&black) {
                model.color_left = Color::srgba(0.77, 0.0, 0.48, model.alpha_left).into();
            } else {
                model.color_left = Color::srgba(0.0, 0.0, 0.0, model.alpha_left).into();
            }
        }
        KeyCode::Digit5 => {
            if model.color_right.eq(&black) {
                model.color_right = Color::srgba(0.34, 0.13, 0.5, model.alpha_right).into();
            } else {
                model.color_right = Color::srgba(0.0, 0.0, 0.0, model.alpha_right).into();
            }
        }
        KeyCode::Digit6 => {
            if model.alpha_left == 1.0 {
                model.alpha_left = 0.5;
            } else {
                model.alpha_left = 1.0;
            }
            model.color_left.alpha = model.alpha_left;
        }
        KeyCode::Digit7 => {
            if model.alpha_right == 1.0 {
                model.alpha_right = 0.5;
            } else {
                model.alpha_right = 1.0;
            }
            model.color_right.alpha = model.alpha_right;
        }
        KeyCode::Digit0 => {
            model.act_stroke_cap = LineCap::Round;
            model.alpha_left = 1.0;
            model.alpha_right = 1.0;
            model.color_left = Color::srgba(0.0, 0.0, 0.0, model.alpha_left).into();
            model.color_right = Color::srgba(0.0, 0.0, 0.0, model.alpha_right).into();
        }
        _other_key => {}
    }
}
