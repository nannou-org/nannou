// P_2_1_5_01
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
 * Simple moire effect demonstration by moving, rotating
 * and scaling a shape of densely packed lines over
 * a background also consisting of densely packed lines.
 *
 * MOUSE
 * mouseX              : overlay rotation or position x
 * mouseY              : overlay scaling
 *
 * KEYS
 * 1-2                 : switch draw mode
 * s                   : save png
 *
 * CONTRIBUTED BY
 * [Niels Poldervaart](http://NielsPoldervaart.nl)
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    draw_mode: u8,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(800, 800)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    Model { draw_mode: 1 }
}

fn view(app: &App, model: &Model) {
    // Prepare to draw.
    let mut draw = app.draw();
    draw.background().color(WHITE);

    let win = app.window_rect();
    // first shape (fixed)
    overlay(&draw, &model, win, 3.0);

    // second shape (dynamically translated/rotated and scaled)
    let x = map_range(app.mouse().x, win.left(), win.right(), -50.0, 50.0);
    let a = map_range(app.mouse().x, win.left(), win.right(), -0.5, 0.5);
    let s = map_range(app.mouse().x, win.top(), win.bottom(), 0.7, 1.0);

    match model.draw_mode {
        1 => draw = draw.rotate(a),
        2 => draw = draw.x_y(x, 0.0),
        _ => (),
    }
    draw = draw.scale(s);

    overlay(&draw, &model, win, 2.0);

}

fn overlay(draw: &Draw, model: &Model, rect: Rect, stroke_weight: f32) {
    let w = rect.w() - 100.0;
    let h = rect.h() - 100.0;

    if model.draw_mode == 1 {
        for i in (0..w as usize).step_by(5) {
            let x = i as f32 - (w / 2.0);
            draw.line()
                .start(pt2(x, -h / 2.0))
                .end(pt2(x, h / 2.0))
                .weight(stroke_weight)
                .color(BLACK);
        }
    } else if model.draw_mode == 2 {
        for i in (0..w as usize).step_by(10) {
            draw.ellipse()
                .x_y(0.0, 0.0)
                .radius(i as f32 / 2.0)
                .no_fill()
                .stroke_weight(stroke_weight)
                .stroke(BLACK);
        }
    }
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        KeyCode::Digit1 => {
            model.draw_mode = 1;
        }
        KeyCode::Digit2 => {
            model.draw_mode = 2;
        }
        _other_key => (),
    }
}
