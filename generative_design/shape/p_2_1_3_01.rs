// P_2_1_3_01
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
 * changing circle amount, size and position in a grid
 *
 * MOUSE
 * position x          : circle amount and size
 * position y          : circle position
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
    tile_count_x: usize,
    tile_count_y: usize,
    tile_width: f32,
    tile_height: f32,
    circle_count: usize,
    end_size: f32,
    end_offset: f32,
    act_random_seed: u64,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .view(view)
        .key_released(key_released)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .build()
        .unwrap();

    let tile_count_x = 10;
    let tile_count_y = 10;
    let win = app.window_rect();
    Model {
        tile_count_x,
        tile_count_y,
        tile_width: win.w() / tile_count_x as f32,
        tile_height: win.h() / tile_count_y as f32,
        circle_count: 0,
        end_size: 0.0,
        end_offset: 0.0,
        act_random_seed: 0,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    let draw = app.draw();
    draw.background().color(WHITE);
    let win = app.window_rect();

    // println!("circle_count {} || end_size {} || end offset {}", model.circle_count, model.end_size, model.end_offset);
    draw.x_y(model.tile_width / 2.0, model.tile_height / 2.0);
    for grid_y in 0..=model.tile_count_y {
        for grid_x in 0..=model.tile_count_x {
            let mut draw = draw.x_y(
                win.left() + model.tile_width * grid_x as f32,
                win.top() - model.tile_height * grid_y as f32,
            );
            //println!("x {} || y {}", win.left() + model.tile_width * grid_x as f32, win.top() - model.tile_height * grid_y as f32);
            let scale = model.tile_width / model.tile_height;
            draw = draw.scale(scale);
            let toggle = rng.gen_range(0..4);
            let rotation = match toggle {
                0 => -(PI / 2.0),
                1 => 0.0,
                2 => PI / 2.0,
                3 => PI,
                _ => unreachable!(),
            };
            draw = draw.rotate(rotation);

            // draw module
            for i in 0..model.circle_count {
                let radius =
                    map_range(i, 0, model.circle_count, model.tile_width, model.end_size) / 2.0;
                let offset = map_range(i, 0, model.circle_count, 0.0, model.end_offset);
                draw.ellipse()
                    .x_y(offset, 0.0)
                    .radius(radius)
                    //.color(BLACK);
                    .no_fill()
                    .stroke_weight(1.0 / scale)
                    .stroke(rgba(0.0, 0.0, 0.0, 0.5));
            }
        }
    }

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}

fn mouse_moved(app: &App, model: &mut Model, pos: Point2) {
    let win = app.window_rect();
    model.circle_count = map_range(pos.x, win.left(), win.right(), 1, 30);

    model.end_size = map_range(pos.x, win.left(), win.right(), model.tile_width / 2.0, 0.0);
    model.end_offset = map_range(
        pos.y,
        win.bottom(),
        win.top(),
        0.0,
        (model.tile_width - model.end_size) / 2.0,
    );
}

fn key_released(app: &App, _model: &mut Model, key: Key) {
    if key == Key::S {
        app.main_window()
            .capture_frame(app.exe_name().unwrap() + ".png");
    }
}
