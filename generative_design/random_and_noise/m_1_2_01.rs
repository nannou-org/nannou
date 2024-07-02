// M_1_2_01
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
 * order vs random!
 * how to interpolate between a free composition (random) and a circle shape (order)
 *
 * MOUSE
 * position x          : fade between random and circle shape
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
    act_random_seed: u64,
    count: usize,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(800, 800)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_pressed(key_pressed)
        .build();

    Model {
        act_random_seed: 0,
        count: 150,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);
    let fader_x = map_range(app.mouse().x, win.left(), win.right(), 0.0, 1.0);

    let mut rng = StdRng::seed_from_u64(model.act_random_seed);

    let angle = deg_to_rad(360.0 / model.count as f32);

    for i in 0..model.count {
        // positions
        let random_x = rng.gen_range(win.left()..win.right() + 1.0);
        let random_y = rng.gen_range(win.bottom()..win.top() + 1.0);
        let circle_x = (angle * i as f32).cos() * 300.0;
        let circle_y = (angle * i as f32).sin() * 300.0;

        let x = nannou::geom::Range::new(random_x, circle_x).lerp(fader_x);
        let y = nannou::geom::Range::new(random_y, circle_y).lerp(fader_x);

        draw.ellipse()
            .x_y(x, y)
            .w_h(11.0, 11.0)
            .srgb_u8(0, 130, 163);
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
