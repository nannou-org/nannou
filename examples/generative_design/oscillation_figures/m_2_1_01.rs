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

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    point_count: u64,
    freq: f32,
    phi: f32,
    angle: f32,
    x: f32,
    y: f32,
    do_draw_animation: bool,
}

fn model(_app: &App) -> Model {
    let point_count = 20;
    let freq = 1.0;
    let phi = 0.0;
    let angle = 0.0;
    let x = 0.0;
    let y = 0.0;
    let do_draw_animation = true;

    Model {
        point_count,
        freq,
        phi,
        angle,
        x,
        y,
        do_draw_animation,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    for i in 0..model.point_count {
        model.angle = map_range(i, 0, model.point_count, 0.0, TAU);
        model.y = (model.angle * model.freq + deg_to_rad(model.phi)).sin();
        model.y *= 100.0;
    }

    if model.do_draw_animation {
        model.point_count = app.window_rect().w() as u64 - 250;

        let t = fmod(
            (app.elapsed_frames() as f64 / model.point_count as f64),
            1.0,
        );
        model.angle = map_range(t, 0.0, 1.0, 0.0, TAU);
        model.x = (model.angle * model.freq + deg_to_rad(model.phi));
        model.x *= 100.0 - 125.0;
        model.y = (model.angle * model.freq + deg_to_rad(model.phi));
        model.y = model.y * 100.0;
    } else {
        model.point_count = app.window_rect().w() as u64;
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();

    draw.background().color(WHITE);

    // draw oscillator curve
    let vertices = (0..model.point_count)
        // A sine wave mapped to the range of the window.
        .map(|i| pt2(i as f32, model.y))
        .enumerate()
        // Colour each vertex uniquely based on its index.
        .map(|(i, p)| {
            let rgba = nannou::color::Rgba::new(0.0, 0.0, 0.0, 1.0);
            geom::vertex::Rgba(p, rgba)
        });

    // Draw the polyline.
    draw.polyline().vertices(2.0, vertices);

    if model.do_draw_animation {
        // Circle
        draw.ellipse()
            .x_y(-125.0, 0.0)
            .radius(200.0)
            .rgba(0.0, 0.0, 0.0, 0.5);

        // Lines
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
