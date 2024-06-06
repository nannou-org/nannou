// P_2_2_6_01
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
 * A chain of linked pendulums. Each a little shorter and faster than the one it's linked to.
 * Each joint of the pendulum leaves behind its own trail.
 *
 * KEYS
 * 1                   : toggle pendulum
 * 2                   : toggle pendulum path
 * -                   : decrease speed relation
 * +                   : increase speed relation
 * arrow down          : decrease length of lines
 * arrow up            : increase length of lines
 * arrow left          : decrease joints
 * arrow right         : increase joints
 * del, backspace      : clear screen
 * s                   : save png
 *
 * CONTRIBUTED BY
 * [Niels Poldervaart](http://NielsPoldervaart.nl)
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    joints: usize,
    line_length: f32,
    speed_relation: f32,
    center: Point2,
    pendulum_paths: Vec<Vec<Point2>>,
    start_positions: Vec<Point2>,
    angle: f32,
    max_angle: f32,
    speed: f32,
    show_pendulum: bool,
    show_pendulum_path: bool,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .key_released(key_released)
        .build();

    let joints = 5;
    Model {
        joints,
        line_length: 100.0,
        speed_relation: 2.0,
        center: vec2(0.0, 0.0),
        pendulum_paths: vec![Vec::new(); joints],
        start_positions: vec![pt2(0.0, 0.0); joints],
        angle: 0.0,
        max_angle: 360.0,
        speed: 1.0,
        show_pendulum: true,
        show_pendulum_path: true,
    }
}

fn start_drawing(model: &mut Model) {
    model.start_positions = vec![pt2(0.0, 0.0); model.joints];
    model.pendulum_paths = vec![Vec::new(); model.joints];
    // new empty array for each joint
    for i in 0..model.pendulum_paths.len() {
        model.pendulum_paths[i].clear();
    }
    model.angle = 0.0;
    model.speed = 8.0 / 1.75.powf(model.joints as f32 - 1.0) / 2.0.powf(model.speed_relation - 1.0);
}

fn update(_app: &App, model: &mut Model) {
    model.angle += model.speed;

    // each frame, create new positions for each joint
    if model.angle <= model.max_angle + model.speed {
        // start at the center position
        let mut pos = model.center;

        for i in 0..model.joints {
            let mut a = model.angle * model.speed_relation.powf(i as f32);
            if i % 2 == 1 {
                a = -a;
            }
            let vx = a.to_radians().cos();
            let vy = a.to_radians().sin();
            let mut next_pos = pt2(vx, vy);

            let magnitude = (model.joints - i) as f32 / model.joints as f32 * model.line_length;
            next_pos = next_pos.normalize() * magnitude;
            next_pos += pos;

            model.start_positions[i] = pos;
            model.pendulum_paths[i].push(next_pos);
            pos = next_pos;
        }
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(WHITE);

    if model.angle <= model.max_angle + model.speed {
        for i in 0..model.joints {
            if model.show_pendulum {
                let pos = model.start_positions[i];
                let next_pos = model.pendulum_paths[i].last().unwrap();
                draw.ellipse()
                    .x_y(pos.x, pos.y)
                    .radius(3.0)
                    .rgba(0.0, 0.0, 0.0, 0.5);
                draw.line()
                    .start(pt2(pos.x, pos.y))
                    .end(pt2(next_pos.x, next_pos.y))
                    .rgba(0.0, 0.0, 0.0, 0.5);
            }
        }
    }

    if model.show_pendulum_path {
        let weight = 2.5;
        for i in 0..model.pendulum_paths.len() {
            let hue = map_range(i, 0, model.joints, 0.0, 1.0);
            let hsla = hsla(hue, 0.8, 0.6, 0.5);

            let vertices = model.pendulum_paths[i]
                .iter()
                .map(|p| pt2(p.x, p.y))
                // Colour each vertex uniquely based on its index.
                .map(|p| (p, hsla));

            // Draw the polyline as a stroked path.
            draw.polyline()
                .weight(weight)
                .join_round()
                .points_colored(vertices);
        }
    }

}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        KeyCode::ArrowUp => {
            model.line_length += 2.0;
            start_drawing(model);
        }
        KeyCode::ArrowDown => {
            model.line_length -= 2.0;
            start_drawing(model);
        }
        KeyCode::ArrowLeft=> {
            if model.joints > 1 {
                model.joints -= 1;
                start_drawing(model);
            }
        }
        KeyCode::ArrowRight => {
            if model.joints < 10 {
                model.joints += 1;
                start_drawing(model);
            }
        }
        KeyCode::Equals => {
            if model.speed_relation < 5.0 {
                model.speed_relation += 0.5;
                start_drawing(model);
            }
        }
        KeyCode::Minus => {
            if model.speed_relation > 2.0 {
                model.speed_relation -= 0.5;
                start_drawing(model);
            }
        }
        KeyCode::Digit1 => {
            model.show_pendulum = !model.show_pendulum;
        }
        KeyCode::Digit2 => {
            model.show_pendulum_path = !model.show_pendulum_path;
        }
        _other_key => {}
    }
}
