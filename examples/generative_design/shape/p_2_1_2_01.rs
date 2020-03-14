// P_2_1_2_01
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
 * changing size and position of circles in a grid
 *
 * MOUSE
 * position x          : circle position
 * position y          : circle size
 * left click          : random position
 *
 * KEYS
 * s                   : save png
 */
use nannou::prelude::*;
use nannou::rand::{Rng, SeedableRng};

use rand::rngs::StdRng;

fn main() {
    nannou::app(model).run();
}

struct Model {
    tile_count: u32,
    act_random_seed: u64,
    circle_color: Rgba,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let circle_alpha = 0.5;

    Model {
        tile_count: 20,
        act_random_seed: 0,
        circle_color: rgba(0.0, 0.0, 0.0, circle_alpha),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    for grid_y in 0..model.tile_count {
        for grid_x in 0..model.tile_count {
            let tile_w = win.w() / model.tile_count as f32;
            let tile_h = win.h() / model.tile_count as f32;
            let pos_x = (win.left() + (tile_w / 2.0)) + tile_w * grid_x as f32;
            let pos_y = (win.top() - (tile_h / 2.0)) - tile_h * grid_y as f32;

            let mx = clamp(win.right() + app.mouse.x, 0.0, win.w());
            let my = clamp(win.top() - app.mouse.y, 0.0, win.h());

            let shift_x = rng.gen_range(-mx, mx + 1.0) / 20.0;
            let shift_y = rng.gen_range(-mx, mx + 1.0) / 20.0;

            draw.ellipse()
                .x_y(pos_x + shift_x, pos_y + shift_y)
                .radius(my / 15.0)
                .no_fill()
                .stroke(model.circle_color)
                .stroke_weight(my / 50.0);
        }
    }

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}

fn key_pressed(app: &App, _model: &mut Model, key: Key) {
    if key == Key::S {
        app.main_window()
            .capture_frame(app.exe_name().unwrap() + ".png");
    }
}
