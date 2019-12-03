// M_2_2_01
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
 * draws a lissajous curve
 *
 * KEYS
 * a                 : toggle oscillation animation
 * 1/2               : frequency x -/+
 * 3/4               : frequency y -/+
 * arrow left/right  : phi -/+
 * s                 : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    point_count: usize,
    freq_x: f32,
    freq_y: f32,
    phi: f32,
    angle: f32,
    x: f32,
    y: f32,
    do_draw_animation: bool,
    margin: f32,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .with_dimensions(600, 600)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    Model {
        point_count: 600,
        freq_x: 1.0,
        freq_y: 2.0,
        phi: 90.0,
        angle: 0.0,
        x: 0.0,
        y: 0.0,
        do_draw_animation: true,
        margin: 25.0,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let t = fmod(app.elapsed_frames() as f32 / model.point_count as f32, 1.0);
    model.angle = map_range(t, 0.0, 1.0, 0.0, TAU);
    model.x = (model.angle * model.freq_x + deg_to_rad(model.phi)).sin();
    model.x *= app.window_rect().w() / 4.0 - model.margin;
    model.y = (model.angle * model.freq_y).sin();
    model.y *= app.window_rect().h() / 4.0 - model.margin;
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(WHITE);

    let (xs, ys, factor_x, factor_y) = if model.do_draw_animation {
        (
            win.left() + (win.w() * 3.0 / 4.0),
            win.top() - (win.h() * 3.0 / 4.0),
            win.w() / 4.0 - model.margin,
            win.h() / 4.0 - model.margin,
        )
    } else {
        (
            0.0,
            0.0,
            win.w() / 2.0 - model.margin,
            win.h() / 2.0 - model.margin,
        )
    };

    let vertices = (0..=model.point_count)
        .map(|i| {
            let angle = map_range(i, 0, model.point_count, 0.0, TAU);
            let mut x = (angle * model.freq_x + deg_to_rad(model.phi)).sin();
            let mut y = (angle * model.freq_y).sin();
            x *= factor_x;
            y *= factor_y;
            pt2(xs + x, ys + y)
        })
        .enumerate()
        .map(|(_i, p)| {
            let rgba = rgba(0.0, 0.0, 0.0, 1.0);
            (p, rgba)
        });

    // Draw the sine wave.
    draw.polyline().weight(2.0).colored_points(vertices);

    if model.do_draw_animation {
        // draw x oscillator
        let vertices = (0..model.point_count)
            .map(|i| {
                let angle = map_range(i, 0, model.point_count, 0.0, TAU);
                let mut x = (angle * model.freq_x + deg_to_rad(model.phi)).sin();
                x *= win.w() / 4.0 - model.margin;
                let y = -win.h() * 2.0 / 3.0 - model.margin
                    + i as f32 / model.point_count as f32 * (win.h() / 2.0 - 2.0 * model.margin);
                pt2(xs + x, ys - y)
            })
            .enumerate()
            .map(|(_i, p)| {
                let rgba = rgba(0.0, 0.0, 0.0, 1.0);
                (p, rgba)
            });

        draw.polyline().weight(2.0).colored_points(vertices);

        // draw y oscillator
        let vertices = (0..model.point_count)
            .map(|i| {
                let angle = map_range(i, 0, model.point_count, 0.0, TAU);
                let mut y = (angle * model.freq_y).sin();
                y *= win.h() / 4.0 - model.margin;
                let x = -win.w() * 2.0 / 3.0 - model.margin
                    + i as f32 / model.point_count as f32 * (win.w() / 2.0 - 2.0 * model.margin);
                pt2(xs + x, ys - y)
            })
            .enumerate()
            .map(|(_i, p)| {
                let rgba = rgba(0.0, 0.0, 0.0, 1.0);
                (p, rgba)
            });
        draw.polyline().weight(2.0).colored_points(vertices);

        let osc_yx = -win.w() * 2.0 / 3.0 - model.margin
            + fmod(model.angle / TAU, 1.0) * (win.w() / 2.0 - 2.0 * model.margin);
        let osc_xy = -win.h() * 2.0 / 3.0 - model.margin
            + fmod(model.angle / TAU, 1.0) * (win.h() / 2.0 - 2.0 * model.margin);

        let c = rgba(0.0, 0.0, 0.0, 0.5);
        draw.line()
            .start(pt2(xs + model.x, ys - osc_xy))
            .end(pt2(xs + model.x, ys - model.y))
            .color(c);
        draw.line()
            .start(pt2(xs + osc_xy, ys - model.y))
            .end(pt2(xs + model.x, ys - model.y))
            .color(c);

        let c = rgba(0.0, 0.0, 0.0, 1.0);
        draw.ellipse()
            .x_y(xs + model.x, ys - osc_xy)
            .radius(4.0)
            .stroke(gray(1.0))
            .stroke_weight(2.0)
            .color(c);
        draw.ellipse()
            .x_y(xs + osc_yx, ys - model.y)
            .radius(4.0)
            .stroke(gray(1.0))
            .stroke_weight(2.0)
            .color(c);

        draw.ellipse()
            .x_y(xs + model.x, ys - model.y)
            .radius(5.0)
            .stroke(gray(1.0))
            .stroke_weight(2.0)
            .color(c);
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
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
    model.freq_x = model.freq_x.max(1.0);
    model.freq_y = model.freq_y.max(1.0);
}
