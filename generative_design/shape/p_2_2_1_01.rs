// P_2_2_1_01
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
 * draw the path of a stupid agent
 *
 * MOUSE
 * position x          : drawing speed
 *
 * KEYS
 * r                   : clear display
 * s                   : save png
 */
use nannou::prelude::*;

enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    pos_x: f32,
    pos_y: f32,
    positions: Vec<Point2>,
    step_size: f32,
    direction: Direction,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(600, 600)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        pos_x: 0.0,
        pos_y: 0.0,
        positions: vec![pt2(0.0, 0.0); 600],
        step_size: 1.0,
        direction: Direction::North,
    }
}

fn update(app: &App, model: &mut Model) {
    let win = app.window_rect();
    let mx = map_range(app.mouse().x, win.left(), win.right(), 0.0, win.w()) as usize;
    model.positions.resize(mx + 1, pt2(0.0, 0.0));
    for i in 0..=mx {
        let r = random_range(0, 8);
        model.direction = match r {
            0 => Direction::North,
            1 => Direction::NorthEast,
            2 => Direction::East,
            3 => Direction::SouthEast,
            4 => Direction::South,
            5 => Direction::SouthWest,
            6 => Direction::West,
            7 => Direction::NorthWest,
            _ => Direction::North,
        };
        match model.direction {
            Direction::North => model.pos_y -= model.step_size,
            Direction::NorthEast => {
                model.pos_x += model.step_size;
                model.pos_y -= model.step_size
            }
            Direction::East => {
                model.pos_x += model.step_size;
            }
            Direction::SouthEast => {
                model.pos_x += model.step_size;
                model.pos_y += model.step_size
            }
            Direction::South => model.pos_y += model.step_size,
            Direction::SouthWest => {
                model.pos_x -= model.step_size;
                model.pos_y += model.step_size
            }
            Direction::West => {
                model.pos_x -= model.step_size;
            }
            Direction::NorthWest => {
                model.pos_x -= model.step_size;
                model.pos_y -= model.step_size
            }
        }

        if model.pos_x > win.right() {
            model.pos_x = win.left();
        }
        if model.pos_x < win.left() {
            model.pos_x = win.right();
        }
        if model.pos_y < win.bottom() {
            model.pos_y = win.top();
        }
        if model.pos_y > win.top() {
            model.pos_y = win.bottom();
        }

        model.positions[i] = pt2(model.pos_x, model.pos_y);
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

    if app.elapsed_frames() == 0 || app.keys().just_pressed(KeyCode::KeyR) {
        draw.background().color(WHITE);
    }
    model.positions.iter().for_each(|pos| {
        draw.ellipse()
            .x_y(pos.x + model.step_size / 2.0, pos.y + model.step_size / 2.0)
            .radius(1.0)
            .rgba(0.0, 0.0, 0.0, 0.15);
    });
}

fn key_released(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window()
            .save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
