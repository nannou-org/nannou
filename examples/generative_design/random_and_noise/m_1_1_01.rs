// M_1_1_01
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
 * draws a random chart and shows how to use randomSeed.
 *
 * MOUSE
 * click               : new random line
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
    act_random_seed: u64,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .dimensions(1024, 256)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .build()
        .unwrap();

    Model {
        act_random_seed: 42,
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    let range = win.w() as usize / 10;
    let vertices = (0..=range).map(|i| {
        let y = rng.gen_range(win.bottom(), win.top() + 1.0) as f32;
        pt2(win.left() + (i as f32 * 10.0), y as f32)
    });
    draw.polyline()
        .weight(1.0)
        .points(vertices)
        .rgb(0.0, 0.5, 0.64);

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    for x in (0..win.w() as usize).step_by(10) {
        let y = rng.gen_range(win.bottom(), win.top() + 1.0) as f32;
        draw.ellipse()
            .x_y(win.left() + x as f32, y as f32)
            .w_h(3.0, 3.0)
            .color(BLACK);
    }
    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}
