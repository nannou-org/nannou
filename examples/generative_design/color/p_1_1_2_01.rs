// P_1_1_2_01
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
 * changing the color circle by moving the mouse.
 *
 * MOUSE
 * position x          : saturation
 * position y          : brighness
 *
 * KEYS
 * 1-5                 : number of segments
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    segment_count: usize,
    radius: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(720, 720)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    Model {
        segment_count: 360,
        radius: 300.0,
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => {
            model.segment_count = 360;
        }
        Key::Key2 => {
            model.segment_count = 45;
        }
        Key::Key3 => {
            model.segment_count = 24;
        }
        Key::Key4 => {
            model.segment_count = 12;
        }
        Key::Key5 => {
            model.segment_count = 6;
        }
        _other_key => {}
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();

    draw.background().hsv(1.0, 0.0, 1.0);

    let angle_step = 360 / model.segment_count;

    for angle in (0..360).step_by(angle_step) {
        let mut points = Vec::new();
        points.push(pt2(0.0, 0.0));

        let vx = (angle as f32).to_radians().cos() * model.radius;
        let vy = (angle as f32).to_radians().sin() * model.radius;
        points.push(pt2(vx, vy));

        let next_vx = ((angle + angle_step) as f32).to_radians().cos() * model.radius;
        let next_vy = ((angle + angle_step) as f32).to_radians().sin() * model.radius;
        points.push(pt2(next_vx, next_vy));

        let mx = (app.mouse.x / app.window_rect().w()) + 0.5;
        let my = (app.mouse.y / app.window_rect().h()) + 0.5;

        draw.polygon()
            .stroke(hsv(angle as f32 / 360.0, my, mx))
            .hsv(angle as f32 / 360.0, mx, my)
            .points(points);
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
