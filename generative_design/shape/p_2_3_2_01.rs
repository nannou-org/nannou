// P_2_3_2_01
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
 * draw tool. shows how to work with relations between elements.
 *
 * MOUSE
 * drag                : draw
 *
 * KEYS
 * 1                   : draw mode 1 - fixed distance
 * 2                   : draw mode 2 - distance threshold
 * del, backspace      : clear screen
 * arrow up            : line length +
 * arrow down          : line length -
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    draw_mode: u8,
    col: Srgba,
    x: f32,
    y: f32,
    step_size: f32,
    line_length: f32,
    angle: f32,
    dist: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        draw_mode: 1,
        col: rgba(random_f32(), random_f32(), random_f32(), random_f32() * 0.4),
        x: 0.0,
        y: 0.0,
        step_size: 5.0,
        line_length: 25.0,
        angle: 0.0,
        dist: 0.0,
    }
}

fn update(app: &App, model: &mut Model) {
    if app.mouse.buttons.left().is_down() {
        model.dist = pt2(model.x, model.y).distance(pt2(app.mouse().x, app.mouse().y));

        if model.dist > model.step_size {
            model.angle = (app.mouse().y - model.y).atan2(app.mouse().x - model.x);
            if model.draw_mode == 1 {
                model.x = model.x + model.angle.cos() * model.step_size;
                model.y = model.y + model.angle.sin() * model.step_size;
            } else {
                model.x = app.mouse().x;
                model.y = app.mouse().y;
            }
        }
    }
}

fn view(app: &App, model: &Model) {
    let mut draw = app.draw();
    if app.elapsed_frames() == 0 || app.keys().just_pressed(KeyCode::Delete) {
        draw.background().color(WHITE);
    }

    if model.dist > model.step_size {
        draw = draw.x_y(model.x, model.y).rotate(model.angle);
        let c = if app.elapsed_frames() % 2 == 0 {
            rgba(0.6, 0.6, 0.6, 1.0)
        } else {
            model.col
        };
        draw.line()
            .start(pt2(0.0, 0.0))
            .end(pt2(
                0.0,
                model.line_length * random_range(0.95, 1.0) * model.dist / 10.0,
            ))
            .color(c);
    }

    // Write to the window frame.

}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    model.x = app.mouse().x;
    model.y = app.mouse().y;
    model.col = Color::srgba(random_f32(), random_f32(), random_f32(), random_f32() * 0.4);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        KeyCode::Up => {
            model.line_length += 5.0;
        }
        KeyCode::Down => {
            model.line_length -= 5.0;
        }
        _otherkey => (),
    }
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyS => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        // default colors from 1 to 4
        KeyCode::Digit1 => {
            model.draw_mode = 1;
        }
        KeyCode::Digit2 => {
            model.draw_mode = 2;
        }
        _otherkey => (),
    }
}
