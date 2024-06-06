// P_2_1_2_04
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
 * moving corners of rectangles in a grid
 *
 * MOUSE
 * position x          : corner position offset x
 * position y          : corner position offset y
 * left click          : random position
 *
 * KEYS
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
    rect_size: f32,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .key_pressed(key_pressed)
        .mouse_pressed(mouse_pressed)
        .build();

    Model {
        tile_count: 20,
        act_random_seed: 0,
        rect_size: 30.0,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    for grid_y in 0..model.tile_count {
        for grid_x in 0..model.tile_count {
            let tile_w = win.w() / model.tile_count as f32;
            let tile_h = win.h() / model.tile_count as f32;
            let pos_x = win.left() + tile_w * grid_x as f32;
            let pos_y = (win.top() - tile_h) - tile_h * grid_y as f32;

            let mx = clamp(win.right() + app.mouse().x, 0.0, win.w());
            let my = clamp(win.top() - app.mouse().x, 0.0, win.h());

            let shift_x1 = mx / 20.0 * rng.gen_range(-1.0..1.0);
            let shift_y1 = my / 20.0 * rng.gen_range(-1.0..1.0);
            let shift_x2 = mx / 20.0 * rng.gen_range(-1.0..1.0);
            let shift_y2 = my / 20.0 * rng.gen_range(-1.0..1.0);
            let shift_x3 = mx / 20.0 * rng.gen_range(-1.0..1.0);
            let shift_y3 = my / 20.0 * rng.gen_range(-1.0..1.0);
            let shift_x4 = mx / 20.0 * rng.gen_range(-1.0..1.0);
            let shift_y4 = my / 20.0 * rng.gen_range(-1.0..1.0);
            let mut points = Vec::new();
            points.push(pt2(pos_x + shift_x1, pos_y + shift_y1));
            points.push(pt2(pos_x + model.rect_size + shift_x2, pos_y + shift_y2));
            points.push(pt2(
                pos_x + model.rect_size + shift_x3,
                pos_y + model.rect_size + shift_y3,
            ));
            points.push(pt2(pos_x + shift_x4, pos_y + model.rect_size + shift_y4));

            draw.polygon().hsva(0.53, 1.0, 0.64, 0.7).points(points);
        }
    }

}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}

fn key_pressed(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
