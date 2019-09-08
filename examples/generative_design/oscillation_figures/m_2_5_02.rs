// M_2_5_02
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
 * explore different parameters of drawing lissajous figures
 *
 * KEYS
 * s                   : save png
 */
use nannou::prelude::*;
use nannou::rand::{Rng, SeedableRng};

use rand::rngs::StdRng;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    point_count: usize,
    point_index: usize,
    lissajous_points: Vec<Point2>,
    freq_x: f32,
    freq_y: f32,
    phi: f32,
    mod_freq_x: f32,
    mod_freq_y: f32,
    mod_freq_2_strength: f32,
    random_offset: f32,
    invert_background: bool,
    line_weight: f32,
    line_alpha: f32,
    connect_all_points: bool,
    connection_radius: f32,
    min_hue_value: f32,
    max_hue_value: f32,
    saturation_value: f32,
    brightness_value: f32,
    invert_hue: bool,
    should_draw: bool,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .with_dimensions(800, 800)
        .view(view)
        .build()
        .unwrap();

    let lissajous_points = Vec::new();

    let mut model = Model {
        point_count: 1500,
        point_index: 0,
        lissajous_points,
        freq_x: 13.0,
        freq_y: 11.0,
        phi: 97.0,
        mod_freq_x: 0.0,
        mod_freq_y: 0.0,
        mod_freq_2_strength: 0.0,
        random_offset: 2.0,
        invert_background: true,
        line_weight: 2.0,
        line_alpha: 0.3,
        connect_all_points: true,
        connection_radius: 120.0,
        min_hue_value: 0.0,
        max_hue_value: 1.0,
        saturation_value: 0.8,
        brightness_value: 0.0,
        invert_hue: false,
        should_draw: true,
    };

    calculate_lissajous_points(app, &mut model);

    model
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.point_index += 1;

    if model.point_index >= model.point_count - 1 {
        model.should_draw = false;
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    let draw = app.draw();

    if app.elapsed_frames() == 1 {
        if model.invert_background {
            draw.background().color(BLACK);
        } else {
            draw.background().color(WHITE);
        }
    }
    if model.should_draw {
        if !model.connect_all_points {
            for i in 0..(model.point_count - 1) {
                draw_line(
                    &draw,
                    &model,
                    model.lissajous_points[i],
                    model.lissajous_points[i + 1],
                );
            }
        } else {
            for i2 in 0..model.point_index {
                draw_line(
                    &draw,
                    &model,
                    model.lissajous_points[model.point_index],
                    model.lissajous_points[i2],
                );
            }
        }
    }
    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn draw_line(draw: &app::Draw, model: &Model, p1: Point2, p2: Point2) {
    let dist = p1.distance(p2);
    let angle = (1.0 / (dist / model.connection_radius + 1.0)).powf(6.0);

    if dist <= model.connection_radius {
        let lerp = if model.invert_hue { 1.0 - angle } else { angle };
        let hue = map_range(lerp, 0.0, 1.0, model.min_hue_value, model.max_hue_value);

        let b = if model.invert_background {
            1.0
        } else {
            model.brightness_value
        };
        let c = hsva(
            hue,
            model.saturation_value,
            b,
            (angle * model.line_alpha + (model.point_index % 2 * 2) as f32) / 100.0,
        );
        draw.line()
            .start(p1)
            .end(p2)
            .stroke_weight(model.line_weight)
            .color(c);
    }
}

fn calculate_lissajous_points(app: &App, model: &mut Model) {
    let win = app.window_rect();
    model.lissajous_points.clear();

    let mut rng = StdRng::seed_from_u64(0);

    for i in 0..=model.point_count {
        let angle = map_range(i, 0, model.point_count, 0.0, TAU);

        let fmx = (angle * model.mod_freq_x).sin() * model.mod_freq_2_strength + 1.0;
        let fmy = (angle * model.mod_freq_y).sin() * model.mod_freq_2_strength + 1.0;

        let mut x = (angle * model.freq_x * fmx + deg_to_rad(model.phi)).sin()
            * (angle * model.mod_freq_x).cos();
        let mut y = (angle * model.freq_y * fmy).sin() * (angle * model.mod_freq_y).cos();

        let rx = rng.gen_range(-model.random_offset, model.random_offset + 1.0);
        let ry = rng.gen_range(-model.random_offset, model.random_offset + 1.0);

        x = (x * (win.w() / 2.0 - 30.0 - model.random_offset) + win.w() / 2.0) + rx;
        y = (y * (win.h() / 2.0 - 30.0 - model.random_offset) + win.h() / 2.0) + ry;
        model
            .lissajous_points
            .push(pt2(win.left() + x, win.top() - y));
    }
}
