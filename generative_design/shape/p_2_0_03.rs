// P_2_0_03
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
 * drawing with a changing shape by draging the mouse.
 *
 * MOUSE
 * position x          : length
 * position y          : thickness and number of lines
 * drag                : draw
 *
 * KEYS
 * 1-3                 : stroke color
 * spacebar            : erase
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    clicked: bool,
    clear_background: bool,
    stroke_color: Hsva,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(720, 720)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build()
        .unwrap();
    Model {
        clicked: false,
        clear_background: false,
        stroke_color: hsva(0.0, 0.0, 0.0, 0.1),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Prepare to draw.
    let draw = app.draw();
    let win = app.window_rect();
    let circle_resolution = map_range(app.mouse.y, win.top(), win.bottom(), 3, 10);
    let radius = app.mouse.x - win.left();
    let angle = TAU / circle_resolution as f32;

    if app.elapsed_frames() == 1 || model.clear_background {
        draw.background().color(WHITE);
    }

    let mut points = Vec::new();
    for i in 0..circle_resolution {
        let x = (angle * i as f32).cos() * radius;
        let y = (angle * i as f32).sin() * radius;
        points.push(pt2(x, y));
    }

    if model.clicked {
        draw.polygon()
            .stroke(model.stroke_color)
            .stroke_weight(2.0)
            .no_fill()
            .points(points);
    }
    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.clicked = true;
}
fn mouse_released(_app: &App, model: &mut Model, _button: MouseButton) {
    model.clicked = false;
}
fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            model.clear_background = true;
        }
        Key::S => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
fn key_released(_app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        model.clear_background = false;
    }
    if key == Key::Key1 {
        model.stroke_color = hsva(0.0, 0.0, 0.0, 0.1);
    }
    if key == Key::Key2 {
        model.stroke_color = hsva(0.53, 1.0, 0.64, 0.1);
    }
    if key == Key::Key3 {
        model.stroke_color = hsva(0.147, 1.0, 0.71, 0.1);
    }
}
