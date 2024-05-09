// P_2_3_3_01
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
 * draw tool. shows how to draw with dynamic elements.
 *
 * MOUSE
 * drag                : draw with text
 *
 * KEYS
 * del, backspace      : clear screen
 * arrow up            : angle distortion +
 * arrow down          : angle distortion -
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    x: f32,
    y: f32,
    step_size: f32,
    letters: String,
    font_size: u32,
    font_size_min: u32,
    angle: f32,
    angle_distortion: f32,
    distance: f32,
    counter: usize,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build();

    let letters = "All the world's a stage, and all the men and women merely players. They have their exits and their entrances.".to_string();
    Model {
        x: 0.0,
        y: 0.0,
        step_size: 5.0,
        letters,
        font_size: 3,
        font_size_min: 3,
        angle: 0.0,
        angle_distortion: 0.0,
        distance: 0.0,
        counter: 0,
    }
}

fn update(app: &App, model: &mut Model) {
    if app.mouse_buttons().just_pressed(MouseButton::Left) {
        model.distance = pt2(model.x, model.y).distance(pt2(app.mouse().x, app.mouse().y));
        model.font_size = model.font_size_min + model.distance as u32 / 2;

        let win_rect = app.main_window().rect();
        let letter = &model
            .letters
            .chars()
            .nth(model.counter)
            .unwrap()
            .to_string();
        model.step_size = text(letter)
            .font_size(model.font_size)
            .build(win_rect)
            .bounding_rect()
            .w();

        if model.distance > model.step_size {
            model.angle = (app.mouse().y - model.y).atan2(app.mouse().x - model.x);
            model.counter += 1;
            if model.counter >= model.letters.len() {
                model.counter = 0;
            }

            model.x = model.x + model.angle.cos() * model.step_size;
            model.y = model.y + model.angle.sin() * model.step_size;
        }
    }
}

fn view(app: &App, model: &Model) {
    if app.elapsed_frames() == 0 {
        draw.background().color(WHITE);
    }

    let draw = app.draw();

    if model.distance > model.step_size {
        let draw = app
            .draw()
            .x_y(model.x, model.y)
            .rotate(model.angle + (random_f32() * model.angle_distortion));

        let letter = &model
            .letters
            .chars()
            .nth(model.counter)
            .unwrap()
            .to_string();
        draw.text(letter)
            .font_size(model.font_size)
            .x_y(0.0, 0.0)
            .color(BLACK);
    }

    // Write to the window frame.

}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    model.x = app.mouse().x;
    model.y = app.mouse().y;
}

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    if key == KeyCode::ArrowUp {
        model.angle_distortion += 0.1;
    }
    if key == KeyCode::ArrowDown {
        model.angle_distortion -= 0.1;
    }
}

fn key_released(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
