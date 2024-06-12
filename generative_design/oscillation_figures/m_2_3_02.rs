// M_2_3_02
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
 * draws a modulated lissajous curve
 *
 * MOUSE
 * position x        : number of points
 *
 * KEYS
 * d                 : draw mode
 * 1/2               : frequency x -/+
 * 3/4               : frequency y -/+
 * arrow left/right  : phi -/+
 * 7/8               : modulation frequency x -/+
 * 9/0               : modulation frequency y -/+
 * s                 : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    point_count: usize,
    freq_x: f32,
    freq_y: f32,
    phi: f32,
    mod_freq_x: f32,
    mod_freq_y: f32,
    max_dist: f32,
    draw_mode: u8,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let win = app.window_rect();
    let sq = |v: f32| -> f32 { v * v };

    let max_dist = (sq(win.w() / 2.0 - 50.0) + sq(win.h() / 2.0 - 50.0)).sqrt();
    Model {
        point_count: 500,
        freq_x: 1.0,
        freq_y: 4.0,
        phi: 60.0,
        mod_freq_x: 2.0,
        mod_freq_y: 1.0,
        max_dist,
        draw_mode: 2,
    }
}

fn update(app: &App, model: &mut Model) {
    let mx = clamp(
        app.window_rect().right() + app.mouse().x,
        0.0,
        app.window_rect().w(),
    );
    model.point_count = mx as usize * 2 + 200;
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    if model.draw_mode == 1 {
        let vertices = (0..=model.point_count)
            .map(|i| {
                let angle = map_range(i, 0, model.point_count, 0.0, TAU);
                let mut x = (angle * model.freq_x + deg_to_rad(model.phi)).sin()
                    * (angle * model.mod_freq_x).cos();
                let mut y = (angle * model.freq_y).sin() * (angle * model.mod_freq_y).cos();
                x *= win.w() / 2.0 - 50.0;
                y *= win.h() / 2.0 - 50.0;
                pt2(x, y)
            })
            .enumerate()
            .map(|(_i, p)| {
                let rgba = Color::srgba(0.0, 0.0, 0.0, 1.0);
                (p, rgba)
            });
        draw.polyline().weight(1.0).points_colored(vertices);
    } else if model.draw_mode == 2 {
        for i in 0..=model.point_count {
            let angle = map_range(i, 0, model.point_count, 0.0, TAU);
            let mut x = (angle * model.freq_x + deg_to_rad(model.phi)).sin()
                * (angle * model.mod_freq_x).cos();
            let mut y = (angle * model.freq_y).sin() * (angle * model.mod_freq_y).cos();
            x *= win.w() / 2.0 - 50.0;
            y *= win.h() / 2.0 - 50.0;

            if i > 0 {
                let w = pt2(0.0, 0.0).distance(pt2(x, y));
                let prev_angle = map_range(i - 1, 0, model.point_count, 0.0, TAU);
                let mut old_x = (prev_angle * model.freq_x + deg_to_rad(model.phi)).sin()
                    * (prev_angle * model.mod_freq_x).cos();
                let mut old_y =
                    (prev_angle * model.freq_y).sin() * (prev_angle * model.mod_freq_y).cos();
                old_x *= win.w() / 2.0 - 50.0;
                old_y *= win.h() / 2.0 - 50.0;
                let g = (i % 2 * 2) as f32;
                let c = Color::srgba(g, g, g, map_range(w, 0.0, model.max_dist, 1.0, 0.0));
                draw.line()
                    .start(pt2(old_x, old_y))
                    .end(pt2(x, y))
                    .stroke_weight(8.0)
                    .caps_round()
                    .color(c);
            }
        }
    }
}

fn key_pressed(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyD => {
            if model.draw_mode == 1 {
                model.draw_mode = 2;
            } else {
                model.draw_mode = 1;
            }
        }
        KeyCode::Digit1 => {
            model.freq_x -= 1.0;
        }
        KeyCode::Digit2 => {
            model.freq_x += 1.0;
        }
        KeyCode::Digit3 => {
            model.freq_y -= 1.0;
        }
        KeyCode::Digit4 => {
            model.freq_y += 1.0;
        }
        KeyCode::Digit7 => {
            model.mod_freq_x -= 1.0;
        }
        KeyCode::Digit8 => {
            model.mod_freq_x += 1.0;
        }
        KeyCode::Digit9 => {
            model.mod_freq_y -= 1.0;
        }
        KeyCode::Digit0 => {
            model.mod_freq_y += 1.0;
        }
        KeyCode::ArrowLeft => {
            model.phi -= 15.0;
        }
        KeyCode::ArrowRight => {
            model.phi += 15.0;
        }
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
    model.freq_x = model.freq_x.max(1.0);
    model.freq_y = model.freq_y.max(1.0);
    model.mod_freq_x = model.mod_freq_x.max(1.0);
    model.mod_freq_y = model.mod_freq_y.max(1.0);
}
