// P_2_1_1_01
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
 * changing strokeweight and strokecaps on diagonals in a grid
 *
 * MOUSE
 * position x          : left diagonal strokeweight
 * position y          : right diagonal strokeweight
 * left click          : new random layout
 *
 * KEYS
 * 1                   : round strokecap
 * 2                   : square strokecap
 * 3                   : butt strokecap
 * s                   : save png
 */
use nannou::prelude::*;
use nannou::rand::{Rng, SeedableRng};

use lyon::tessellation::LineCap;
use rand::rngs::StdRng;

fn main() {
    nannou::app(model).run();
}

struct Model {
    tile_count: u32,
    act_random_seed: u64,
    act_stroke_cap: LineCap,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .with_dimensions(600, 600)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        tile_count: 20,
        act_random_seed: 0,
        act_stroke_cap: LineCap::Round,
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Prepare to draw.
    let draw = app.draw();
    draw.background().color(BLACK);
    let win = app.window_rect();

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    for grid_y in 0..model.tile_count {
        for grid_x in 0..model.tile_count {
            let tile_w = win.w() / model.tile_count as f32;
            let tile_h = win.h() / model.tile_count as f32;
            let pos_x = win.left() + tile_w * grid_x as f32;
            let pos_y = (win.top() - tile_h) - tile_h * grid_y as f32;
            let mx = clamp(win.right() + app.mouse.x, 0.0, win.w());
            let my = clamp(win.top() - app.mouse.y, 0.0, win.h());

            let toggle = rng.gen::<bool>();

            if toggle == false {
                draw.line()
                    .weight(mx / 20.0)
                    .caps(model.act_stroke_cap)
                    .points(
                        pt2(pos_x, pos_y),
                        pt2(
                            pos_x + win.w() / model.tile_count as f32,
                            pos_y + win.h() / model.tile_count as f32,
                        ),
                    );
            }
            if toggle == true {
                draw.line()
                    .weight(my / 20.0)
                    .caps(model.act_stroke_cap)
                    .points(
                        pt2(pos_x, pos_y + win.w() / model.tile_count as f32),
                        pt2(pos_x + win.h() / model.tile_count as f32, pos_y),
                    );
            }
        }
    }

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}
fn key_released(_app: &App, model: &mut Model, key: Key) {
    if key == Key::Key1 {
        model.act_stroke_cap = LineCap::Round;
    }
    if key == Key::Key2 {
        model.act_stroke_cap = LineCap::Square;
    }
    if key == Key::Key3 {
        model.act_stroke_cap = LineCap::Butt;
    }
}
