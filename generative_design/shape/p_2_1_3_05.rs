// P_2_1_3_05
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
 * changing positions of stapled circles in a grid
 *
 * MOUSE
 * position x          : horizontal position shift
 * position y          : vertical position shift
 * left click          : random position shift
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
    tile_count_x: u32,
    tile_count_y: u32,
    act_random_seed: u64,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build();

    Model {
        tile_count_x: 10,
        tile_count_y: 10,
        act_random_seed: 0,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);
    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    let tile_width = win.w() / model.tile_count_x as f32;
    let tile_height = win.h() / model.tile_count_y as f32;
    let step_size = clamp(
        map_range(app.mouse().x, win.left(), win.right(), 0.0, win.w()),
        0.0,
        win.w(),
    ) / 10.0;
    let end_size = clamp(
        map_range(app.mouse().y, win.top(), win.bottom(), 0.0, win.h()),
        0.0,
        win.h(),
    ) / 10.0;
    let color_step = 6;

    for grid_y in 0..model.tile_count_y {
        for grid_x in 0..model.tile_count_x {
            let pos_x = (win.left() + (tile_width / 2.0)) + tile_width * grid_x as f32;
            let pos_y = (win.top() - (tile_height / 2.0)) - tile_height * grid_y as f32;

            //modules
            let heading = rng.gen_range(0..4);

            for i in 0..step_size as usize {
                let radius = map_range(i, 0, step_size as usize, tile_width, end_size) / 2.0;
                let col = gray((255.0 - (i * color_step) as f32) / 255.0);
                let (x, y) = match heading {
                    0 => (pos_x + i as f32, pos_y),
                    1 => (pos_x, pos_y + i as f32),
                    2 => (pos_x - i as f32, pos_y),
                    3 => (pos_x, pos_y - i as f32),
                    _ => (0.0, 0.0),
                };
                draw.ellipse()
                    .x_y(x, y)
                    .radius(radius)
                    .resolution(32.0)
                    .color(col);
            }
        }
    }

}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}
