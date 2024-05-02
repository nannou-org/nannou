// M_2_5_01
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
 * draw lissajous figures with all points connected
 *
 * KEYS
 * 1/2               : frequency x -/+
 * 3/4               : frequency y -/+
 * arrow left/right  : phi -/+
 * 7/8               : modulation frequency x -/+
 * 9/0               : modulation frequency y -/+
 * s                 : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    point_count: usize,
    lissajous_points: Vec<Point2>,
    freq_x: f32,
    freq_y: f32,
    phi: f32,
    mod_freq_x: f32,
    mod_freq_y: f32,
    line_weight: f32,
    line_color: Rgba,
    line_alpha: f32,
    connection_radius: f32,
    should_draw: bool,
    should_draw_frame: u64,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(800, 800)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let lissajous_points = Vec::new();
    let line_alpha = 0.2;

    let mut model = Model {
        point_count: 1000,
        lissajous_points,
        freq_x: 4.0,
        freq_y: 7.0,
        phi: 15.0,
        mod_freq_x: 3.0,
        mod_freq_y: 2.0,
        line_weight: 1.5,
        line_color: rgba(0.0, 0.0, 0.0, line_alpha),
        line_alpha,
        connection_radius: 200.0,
        should_draw: true,
        should_draw_frame: 0,
    };

    calculate_lissajous_points(app, &mut model);

    model
}

fn calculate_lissajous_points(app: &App, model: &mut Model) {
    let win = app.window_rect();
    model.lissajous_points.clear();

    for i in 0..=model.point_count {
        let angle = map_range(i, 0, model.point_count, 0.0, TAU);
        let mut x =
            (angle * model.freq_x + deg_to_rad(model.phi)).sin() * (angle * model.mod_freq_x).cos();
        let mut y = (angle * model.freq_y).sin() * (angle * model.mod_freq_y).cos();
        x *= win.w() / 2.0 - 30.0;
        y *= win.h() / 2.0 - 30.0;
        model.lissajous_points.push(pt2(x, y));
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if model.should_draw_frame != app.elapsed_frames() {
        model.should_draw = false;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    if model.should_draw {
        draw.background().color(WHITE);

        for i1 in 0..model.point_count {
            for i2 in 0..i1 {
                let d = model.lissajous_points[i1].distance(model.lissajous_points[i2]);
                let a = (1.0 / (d / model.connection_radius + 1.0)).powf(6.0);

                if d <= model.connection_radius {
                    let mut c = model.line_color;
                    c.alpha = a * model.line_alpha;

                    draw.line()
                        .start(model.lissajous_points[i1])
                        .end(model.lissajous_points[i2])
                        .stroke_weight(model.line_weight)
                        .color(c);
                }
            }
        }
    }
    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => {
            model.freq_x -= 1.0;
        }
        Key::Key2 => {
            model.freq_x += 1.0;
        }
        Key::Key3 => {
            model.freq_y -= 1.0;
        }
        Key::Key4 => {
            model.freq_y += 1.0;
        }
        Key::Key7 => {
            model.mod_freq_x -= 1.0;
        }
        Key::Key8 => {
            model.mod_freq_x += 1.0;
        }
        Key::Key9 => {
            model.mod_freq_y -= 1.0;
        }
        Key::Key0 => {
            model.mod_freq_y += 1.0;
        }
        Key::Left => {
            model.phi -= 15.0;
        }
        Key::Right => {
            model.phi += 15.0;
        }
        Key::S => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
    model.freq_x = model.freq_x.max(1.0);
    model.freq_y = model.freq_y.max(1.0);
    model.mod_freq_x = model.mod_freq_x.max(1.0);
    model.mod_freq_y = model.mod_freq_y.max(1.0);

    calculate_lissajous_points(app, model);

    model.should_draw = true;
    model.should_draw_frame = app.elapsed_frames();
}
