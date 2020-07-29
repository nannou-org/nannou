// P_2_3_1_01
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
 * draw tool. draw with a rotating line.
 *
 * MOUSE
 * drag                : draw
 *
 * KEYS
 * 1-4                 : switch default colors
 * delete/backspace    : clear screen
 * d                   : reverse direction and mirror angle
 * space               : new random color
 * arrow left          : rotation speed -
 * arrow right         : rotation speed +
 * arrow up            : line length +
 * arrow down          : line length -
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    c: Rgba,
    line_length: f32,
    angle: f32,
    angle_speed: f32,
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
        c: rgba(0.7, 0.6, 0.0, 1.0),
        line_length: 0.0,
        angle: 0.0,
        angle_speed: 1.0,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if app.mouse.buttons.left().is_down() {
        model.angle += model.angle_speed;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let mut draw = app.draw();
    if frame.nth() == 0 || app.keys.down.contains(&Key::Delete) {
        frame.clear(WHITE);
    }

    if app.mouse.buttons.left().is_down() {
        draw = draw
            .x_y(app.mouse.x, app.mouse.y)
            .rotate(model.angle.to_radians());
        draw.line()
            .start(pt2(0.0, 0.0))
            .end(pt2(model.line_length, 0.0))
            .color(model.c);
    }

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.line_length = random_range(70.0, 200.0);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Up => {
            model.line_length += 5.0;
        }
        Key::Down => {
            model.line_length -= 5.0;
        }
        Key::Left => {
            model.angle_speed -= 0.5;
        }
        Key::Right => {
            model.angle_speed += 0.5;
        }
        _otherkey => (),
    }
}

fn key_released(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::S => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        // reverse direction and mirror angle
        Key::D => {
            model.angle += 180.0;
            model.angle_speed *= -1.0;
        }
        // change color
        Key::Space => {
            model.c = rgba(
                random_f32(),
                random_f32(),
                random_f32(),
                random_range(0.3, 0.4),
            );
        }
        // default colors from 1 to 4
        Key::Key1 => {
            model.c = rgba(0.7, 0.61, 0.0, 1.0);
        }
        Key::Key2 => {
            model.c = rgba(0.0, 0.5, 0.64, 1.0);
        }
        Key::Key3 => {
            model.c = rgba(0.34, 0.13, 0.5, 1.0);
        }
        Key::Key4 => {
            model.c = rgba(0.77, 0.0, 0.48, 1.0);
        }
        _otherkey => (),
    }
}
