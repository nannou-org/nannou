// P_2_1_2_02
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
 * changing module color and positions in a grid
 *
 * MOUSE
 * position x          : offset x
 * position y          : offset y
 * left click          : random position
 *
 * KEYS
 * 1-3                 : different sets of colors
 * 0                   : default
 * arrow up/down       : background module size
 * arrow left/right    : foreground module size
 * s                   : save png
 */
use nannou::prelude::*;
use nannou::rand::rngs::StdRng;
use nannou::rand::{Rng, SeedableRng};

fn main() {
    nannou::app(model).run();
}

struct Model {
    tile_count: u32,
    act_random_seed: u64,
    module_color_background: Hsva,
    module_color_foreground: Hsva,
    module_alpha_background: f32,
    module_alpha_foreground: f32,
    module_radius_background: f32,
    module_radius_foreground: f32,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build();

    let module_alpha_background = 1.0;
    let module_alpha_foreground = 1.0;

    Model {
        tile_count: 20,
        act_random_seed: 0,
        module_color_background: Hsva::new(0.0, 0.0, 0.0, module_alpha_background),
        module_color_foreground: Hsva::new(0.0, 0.0, 1.0, module_alpha_foreground),
        module_alpha_background,
        module_alpha_foreground,
        module_radius_background: 15.0,
        module_radius_foreground: 7.5,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    let mx = clamp(win.right() + app.mouse().x, 0.0, win.w());
    let my = clamp(win.top() - app.mouse().x, 0.0, win.h());

    for grid_y in 0..model.tile_count {
        for grid_x in 0..model.tile_count {
            let tile_w = win.w() / model.tile_count as f32;
            let tile_h = win.h() / model.tile_count as f32;
            let pos_x = (win.left() + (tile_w / 2.0)) + tile_w * grid_x as f32;
            let pos_y = (win.top() - (tile_h / 2.0)) - tile_h * grid_y as f32;

            let shift_x = rng.gen_range(-mx..mx + 1.0) / 20.0;
            let shift_y = rng.gen_range(-my..my + 1.0) / 20.0;

            draw.ellipse()
                .x_y(pos_x + shift_x, pos_y + shift_y)
                .radius(model.module_radius_background)
                .color(model.module_color_background);
        }
    }

    for grid_y in 0..model.tile_count {
        for grid_x in 0..model.tile_count {
            let tile_w = win.w() / model.tile_count as f32;
            let tile_h = win.h() / model.tile_count as f32;
            let pos_x = (win.left() + (tile_w / 2.0)) + tile_w * grid_x as f32;
            let pos_y = (win.top() - (tile_h / 2.0)) - tile_h * grid_y as f32;

            draw.ellipse()
                .x_y(pos_x, pos_y)
                .radius(model.module_radius_foreground)
                .color(model.module_color_foreground);
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
    match key {
        KeyCode::Digit1 => {
            if model.module_color_background.eq(&Hsva::new(
                0.0,
                0.0,
                0.0,
                model.module_alpha_background,
            )) {
                model.module_color_background =
                    Hsva::new(0.758, 0.73, 0.51, model.module_alpha_background).into();
            } else {
                model.module_color_background =
                    Hsva::new(0.0, 0.0, 0.0, model.module_alpha_background).into();
            }
        }
        KeyCode::Digit2 => {
            if model.module_color_foreground.eq(&Hsva::new(
                1.0,
                1.0,
                1.0,
                model.module_alpha_foreground,
            )) {
                model.module_color_foreground =
                    Hsva::new(0.89, 1.0, 0.77, model.module_alpha_foreground).into();
            } else {
                model.module_color_foreground =
                    Hsva::new(1.0, 1.0, 1.0, model.module_alpha_foreground).into();
            }
        }
        KeyCode::Digit3 => {
            if model.module_alpha_background == 1.0 {
                model.module_alpha_background = 0.5;
                model.module_alpha_foreground = 0.5;
            } else {
                model.module_alpha_background = 1.0;
                model.module_alpha_foreground = 1.0;
            }
            model.module_color_background.alpha = model.module_alpha_background;
            model.module_color_foreground.alpha = model.module_alpha_foreground;
        }
        KeyCode::Digit0 => {
            model.module_radius_background = 15.0;
            model.module_radius_foreground = 7.5;
            model.module_alpha_background = 1.0;
            model.module_alpha_foreground = 1.0;
            model.module_color_background =
                Color::hsva(0.0, 0.0, 0.0, model.module_alpha_background).into();
            model.module_color_foreground =
                Color::hsva(0.0, 0.0, 1.0, model.module_alpha_foreground).into();
        }
        KeyCode::ArrowUp => {
            model.module_radius_background += 2.0;
        }
        KeyCode::ArrowDown => {
            model.module_radius_background = 5.0.max(model.module_radius_background - 2.0);
        }
        KeyCode::ArrowLeft => {
            model.module_radius_foreground = 2.5.max(model.module_radius_foreground - 2.0);
        }
        KeyCode::ArrowRight => {
            model.module_radius_foreground += 2.0;
        }
        _other_key => {}
    }
}
