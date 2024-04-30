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
    nannou::app(model)
        .model_ui()
        .update(update)
        .run();
}

#[derive(Reflect)]
struct Model {
    point_count: usize,
    #[reflect(ignore)]
    lissajous_points: Vec<Point2>,
    freq_x: f32,
    freq_y: f32,
    phi: f32,
    mod_freq_x: f32,
    mod_freq_y: f32,
    line_weight: f32,
    line_color: Srgba,
    line_alpha: f32,
    connection_radius: f32,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .primary()
        .size(800.0, 800.0)
        .view(view)
        .build();

    let lissajous_points = Vec::new();
    let line_alpha = 1.0;

    let mut model = Model {
        point_count: 200,
        lissajous_points,
        freq_x: 4.0,
        freq_y: 7.0,
        phi: 15.0,
        mod_freq_x: 3.0,
        mod_freq_y: 2.0,
        line_weight: 1.5,
        line_color: Srgba::new(0.0, 0.0, 0.0, line_alpha),
        line_alpha,
        connection_radius: 20.0,
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

fn update(app: &App, model: &mut Model) {
    model.phi += 0.1;
    model.mod_freq_x += 0.01;
    model.mod_freq_y += 0.005;
    calculate_lissajous_points(app, model);
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

        draw.background().color(WHITE);

        for i1 in 0..model.point_count {
            for i2 in 0..i1 {
                let d = model.lissajous_points[i1].distance(model.lissajous_points[i2]);
                let avg_x = (model.lissajous_points[i1].x.abs() + model.lissajous_points[i2].x.abs()) / 2.0;
                let a = (1.0 / (d / model.connection_radius + 1.0)).powf(6.0);

                if d <= model.connection_radius  {
                    let mut c = model.line_color;
                    c.with_alpha(a * model.line_alpha);

                    let p1 = model.lissajous_points[i1].abs();
                    let p2 = model.lissajous_points[i2].abs();

                    let line = draw.line()
                        .start(model.lissajous_points[i1])
                        .end(model.lissajous_points[i2]);

                    let emissive_c = Color::srgba(p1.x, p2.y, p2.y, 1.0);

                    line
                        .stroke_weight(model.line_weight)
                        .emissive(emissive_c)
                        .color(c);
                }
            }
        }
}
