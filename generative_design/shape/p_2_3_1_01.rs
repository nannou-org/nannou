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
    c: Srgba,
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
        .build();

    Model {
        c: rgba(0.7, 0.6, 0.0, 1.0),
        line_length: 0.0,
        angle: 0.0,
        angle_speed: 1.0,
    }
}

fn update(app: &App, model: &mut Model) {
    if app.mouse_buttons().just_pressed(MouseButton::Left) {
        model.angle += model.angle_speed;
    }
}

fn view(app: &App, model: &Model) {
    let mut draw = app.draw();
    if app.elapsed_frames() == 0 || app.keys().just_pressed(KeyCode::Delete) {
        draw.background().color(WHITE);
    }

    if app.mouse_buttons().just_pressed(MouseButton::Left) {
        draw = draw
            .x_y(app.mouse().x, app.mouse().y)
            .rotate(model.angle.to_radians());
        draw.line()
            .start(pt2(0.0, 0.0))
            .end(pt2(model.line_length, 0.0))
            .color(model.c);
    }

    // Write to the window frame.

}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.line_length = random_range(70.0, 200.0);
}

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::ArrowUp => {
            model.line_length += 5.0;
        }
        KeyCode::ArrowDown => {
            model.line_length -= 5.0;
        }
        KeyCode::ArrowLeft=> {
            model.angle_speed -= 0.5;
        }
        KeyCode::ArrowRight => {
            model.angle_speed += 0.5;
        }
        _otherkey => (),
    }
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        // reverse direction and mirror angle
        KeyCode::KeyD=> {
            model.angle += 180.0;
            model.angle_speed *= -1.0;
        }
        // change color
        KeyCode::Space => {
            model.c = Color::srgba(
                random_f32(),
                random_f32(),
                random_f32(),
                random_range(0.3, 0.4),
            );
        }
        // default colors from 1 to 4
        KeyCode::Digit1 => {
            model.c = Color::srgba(0.7, 0.61, 0.0, 1.0);
        }
        KeyCode::Digit2 => {
            model.c = Color::srgba(0.0, 0.5, 0.64, 1.0);
        }
        KeyCode::Digit3 => {
            model.c = Color::srgba(0.34, 0.13, 0.5, 1.0);
        }
        KeyCode::Digit4 => {
            model.c = Color::srgba(0.77, 0.0, 0.48, 1.0);
        }
        _otherkey => (),
    }
}
