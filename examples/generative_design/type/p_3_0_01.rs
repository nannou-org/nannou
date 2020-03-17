// P_3_0_01
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
 * changing the size and the position of a letter
 *
 * MOUSE
 * position x          : size
 * position y          : position
 * drag                : draw
 *
 * KEYS
 * a-z                 : change letter
 * ctrl                : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    letter: char,
    mouse_drag: bool,
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
        letter: '8',
        mouse_drag: false,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    if model.mouse_drag == false {
        draw.background().color(WHITE);
    }

    let size = app.mouse.x.max(4.0) as u32 * 5 + 1;
    draw.text(&model.letter.to_string())
        .color(BLACK)
        .font_size(size)
        .x_y(0.0, app.mouse.y);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.mouse_drag = true;
}
fn mouse_released(_app: &App, model: &mut Model, _button: MouseButton) {
    model.mouse_drag = false;
}
fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    //model.letter = key.into();
}
fn key_released(app: &App, _model: &mut Model, key: Key) {
    if key == Key::LControl || key == Key::RControl {
        app.main_window()
            .capture_frame(app.exe_name().unwrap() + ".png");
    }
}
