// P_2_1_2_03
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
 * changing size of circles in a rad grid depending the mouseposition
 *
 * MOUSE
 * position x/y        : module size and offset z
 *
 * KEYS
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    tile_count: u32,
    module_color: Srgba,
    max_distance: f32,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .key_pressed(key_pressed)
        .view(view)
        .build();

    let module_alpha = 0.7;

    Model {
        tile_count: 20,
        module_color: Srgba::new(0.0, 0.0, 0.0, module_alpha),
        max_distance: 500.0,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    for grid_y in 0..model.tile_count {
        for grid_x in 0..model.tile_count {
            let tile_w = win.w() / model.tile_count as f32;
            let tile_h = win.h() / model.tile_count as f32;
            let pos_x = (win.left() + (tile_w / 2.0)) + tile_w * grid_x as f32;
            let pos_y = (win.top() - (tile_h / 2.0)) - tile_h * grid_y as f32;

            let mut diameter = pt2(app.mouse().x, app.mouse().x).distance(pt2(pos_x, pos_y));
            diameter = diameter / model.max_distance * 40.0;

            draw.rect()
                .x_y(pos_x, pos_y)
                .w_h(diameter, diameter)
                .no_fill()
                .stroke(model.module_color)
                .stroke_weight(3.0);
        }
    }
}

fn key_pressed(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window()
            .save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
