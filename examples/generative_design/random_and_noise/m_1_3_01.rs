// M_1_3_01
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
 * draws a chart based on noise values.
 *
 * MOUSE
 * position x          : specify noise input range
 * click               : new noise line
 *
 * KEYS
 * s                   : save png
 */
use nannou::prelude::*;

use nannou::noise::NoiseFn;
use nannou::noise::Seedable;

fn main() {
    nannou::app(model).run();
}

struct Model {
    act_random_seed: u32,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .with_dimensions(1024, 256)
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

    let noise = nannou::noise::Perlin::new().set_seed(model.act_random_seed);

    let noise_x_range = map_range(app.mouse.x, win.left(), win.right(), 0.0, win.w()) / 10.0;

    let range = win.w() as usize / 10;
    let vertices = (0..=range).map(|x| {
        let noise_x = map_range(x as f32, 0.0, win.w(), 0.0, noise_x_range) as f64;
        let y = noise.get([noise_x, noise_x]) * win.h() as f64 / 2.0;
        pt2(win.left() + (x as f32 * 10.0), y as f32)
    });
    draw.polyline()
        .weight(1.0)
        .points(vertices)
        .rgb(0.0, 0.5, 0.64);

    for x in (0..win.w() as usize).step_by(10) {
        let noise_x = map_range(x as f32 / 10.0, 0.0, win.w(), 0.0, noise_x_range) as f64;
        let y = noise.get([noise_x, noise_x]) * win.h() as f64 / 2.0;
        draw.ellipse()
            .x_y(win.left() + x as f32, y as f32)
            .w_h(3.0, 3.0)
            .color(BLACK);
    }
    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u32;
}
