// M_2_1_01
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
 * draws an oscillator
 *
 * KEYS
 * a                 : toggle oscillation animation
 * 1/2               : frequency -/+
 * arrow left/right  : phi -/+
 * s                 : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    point_count: usize,
    freq: f32,
    phi: f32,
    angle: f32,
    x: f32,
    y: f32,
    do_draw_animation: bool,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(800, 400)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    Model {
        point_count: 20,
        freq: 1.0,
        phi: 0.0,
        angle: 0.0,
        x: 0.0,
        y: 0.0,
        do_draw_animation: true,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if model.do_draw_animation {
        model.point_count = app.window_rect().w() as usize - 250;
        let t = fmod(app.elapsed_frames() as f32 / model.point_count as f32, 1.0);
        model.angle = map_range(t, 0.0, 1.0, 0.0, TAU);
        model.x = (model.angle * model.freq + deg_to_rad(model.phi)).cos();
        model.x *= 100.0 - 125.0;
        model.y = (model.angle * model.freq + deg_to_rad(model.phi)).sin();
        model.y = model.y * 100.0;
    } else {
        model.point_count = app.window_rect().w() as usize;
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    let x_start = if model.do_draw_animation {
        win.left() + 250.0
    } else {
        win.left()
    };

    let vertices = (0..model.point_count)
        .map(|i| {
            let angle = map_range(i, 0, model.point_count, 0.0, TAU);
            let mut y = (angle * model.freq + deg_to_rad(model.phi)).sin();
            y *= 100.0;
            pt2(x_start + i as f32, y)
        })
        .enumerate()
        .map(|(_i, p)| {
            let rgba = rgba(0.0, 0.0, 0.0, 1.0);
            (p, rgba)
        });

    // Draw the sine wave.
    draw.polyline().weight(2.0).colored_points(vertices);

    if model.do_draw_animation {
        // Circle
        draw.ellipse()
            .x_y(win.left() + 125.0, 0.0)
            .radius(100.0)
            .stroke(gray(0.0))
            .no_fill();

        // Lines
        let mut c = rgba(0.0, 0.0, 0.0, 0.5);
        draw.line()
            .start(pt2(x_start, -100.0))
            .end(pt2(x_start, 100.0))
            .color(c);
        draw.line()
            .start(pt2(x_start, 0.0))
            .end(pt2(x_start + model.point_count as f32, 0.0))
            .color(c);
        draw.line()
            .start(pt2(x_start - 225.0, 0.0))
            .end(pt2(x_start - 25.0, 0.0))
            .color(c);
        draw.line()
            .start(pt2(x_start - 125.0, -100.0))
            .end(pt2(x_start - 125.0, 100.0))
            .color(c);
        draw.line()
            .start(pt2(x_start + model.x, model.y))
            .end(pt2(x_start - 125.0, 0.0))
            .color(c);

        let t = fmod(app.elapsed_frames() as f32 / model.point_count as f32, 1.0);

        c = rgba(0.0, 0.5, 0.63, 1.0);
        draw.line()
            .start(pt2(x_start + t * model.point_count as f32, model.y))
            .end(pt2(x_start + t * model.point_count as f32, 0.0))
            .stroke_weight(2.0)
            .color(c);
        draw.line()
            .start(pt2(x_start + model.x, model.y))
            .end(pt2(x_start + model.x, 0.0))
            .stroke_weight(2.0)
            .color(c);

        let phi_x = deg_to_rad(model.phi).cos() * 100.0 - 125.0;
        let phi_y = deg_to_rad(model.phi).sin() * 100.0;

        // phi line
        c = rgba(0.0, 0.0, 0.0, 0.5);
        draw.line()
            .start(pt2(x_start - 125.0, 0.0))
            .end(pt2(x_start + phi_x, phi_y))
            .stroke_weight(1.0)
            .color(c);

        // phi dots
        c = rgba(0.0, 0.0, 0.0, 1.0);
        draw.ellipse()
            .x_y(x_start, phi_y)
            .radius(4.0)
            .color(c)
            .stroke(gray(1.0))
            .stroke_weight(2.0);
        draw.ellipse()
            .x_y(x_start + phi_x, phi_y)
            .radius(4.0)
            .color(c)
            .stroke(gray(1.0))
            .stroke_weight(2.0);

        // dot on curve
        draw.ellipse()
            .x_y(x_start + t * model.point_count as f32, model.y)
            .radius(5.0)
            .color(c)
            .stroke(gray(1.0))
            .stroke_weight(2.0);

        // dot on circle
        draw.ellipse()
            .x_y(x_start + model.x, model.y)
            .radius(5.0)
            .color(c)
            .stroke(gray(1.0))
            .stroke_weight(2.0);
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => {
            model.freq -= 1.0;
        }
        Key::Key2 => {
            model.freq += 1.0;
        }
        Key::A => {
            model.do_draw_animation = !model.do_draw_animation;
        }
        Key::Left => {
            model.phi -= 15.0;
        }
        Key::Right => {
            model.phi += 15.0;
        }
        _other_key => {}
    }
    model.freq = model.freq.max(1.0);
}
