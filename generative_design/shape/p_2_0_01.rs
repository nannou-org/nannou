// P_2_0_01
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
 * drawing a filled circle with lines.
 *
 * MOUSE
 * position x          : length
 * position y          : thickness and number of lines
 *
 * KEYS
 * s                   : save png
 */
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::sketch(view).size(550, 550).run();
}

fn view(app: &App) {
    // Prepare to draw.
    let draw = app.draw();
    let win = app.window_rect();
    let circle_resolution = map_range(app.mouse().y, win.top(), win.bottom(), 2, 80);
    let radius = app.mouse().x - win.left();
    let angle = TAU / circle_resolution as f32;

    draw.background().color(BLACK);

    for i in 0..circle_resolution {
        let x = (angle * i as f32).cos() * radius;
        let y = (angle * i as f32).sin() * radius;
        draw.line()
            .start(pt2(0.0, 0.0))
            .end(pt2(x, y))
            .stroke_weight(app.mouse().y / 20.0)
            .caps_round()
            .color(WHITE);
    }

    if app.keys().just_pressed(KeyCode::KeyS) {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
