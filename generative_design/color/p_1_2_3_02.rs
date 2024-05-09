// P_1_2_3_02
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
 * generates a specific color palette and some random "rect-tilings"
 *
 * MOUSE
 * left click          : new composition
 *
 * KEYS
 * s                   : save png
 * c                   : save color palette
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    color_count: usize,
    hue_values: Vec<f32>,
    saturation_values: Vec<f32>,
    brightness_values: Vec<f32>,
    clicked: bool,
    clicked_frame: u64,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1280, 720)
        .view(view)
        .mouse_released(mouse_released)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let color_count = 20;

    // Note you can decalre and pack a vector with random values like this in rust
    let hue_values = vec![0.0; color_count];
    let saturation_values = vec![0.0; color_count];
    let brightness_values = vec![0.0; color_count];

    Model {
        color_count,
        hue_values,
        saturation_values,
        brightness_values,
        clicked: true,
        clicked_frame: 0,
    }
}

fn update(app: &App, model: &mut Model) {
    // Create palette
    for i in 0..model.color_count {
        if i % 2 == 0 {
            model.hue_values[i] = random_f32() * 0.36 + 0.61;
            model.saturation_values[i] = 1.0;
            model.brightness_values[i] = random_f32() * 0.85 + 0.15;
        } else {
            model.hue_values[i] = 0.54;
            model.saturation_values[i] = random_f32() * 0.8 + 0.2;
            model.brightness_values[i] = 1.0;
        }
    }

    if model.clicked_frame != app.elapsed_frames() {
        model.clicked = false;
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

    if model.clicked {
        // ------ area tiling ------
        // count tiles
        let mut counter = 0;
        // row count and row height
        let row_count = random_range(5, 30);
        let row_height = (app.window_rect().h() as i32 / row_count) as i32;

        // seperate each line in parts
        for i in (0..=row_count).rev() {
            // how many fragments
            let mut part_count = i + 1;
            let mut parts = Vec::new();

            for _ in 0..part_count {
                // sub fragment of not?
                if random_f32() < 0.075 {
                    // take care of big values
                    let fragments = random_range(2, 20);
                    part_count = part_count + fragments - 1;
                    for _ in 0..fragments {
                        parts.push(random_range(0, 2));
                    }
                } else {
                    parts.push(random_range(2, 20));
                }
            }

            // add all subparts
            let mut sum_parts_total = 0;
            for ii in 0..part_count {
                sum_parts_total += parts[ii as usize];
            }

            // draw rects
            let mut sum_parts_now = 0;
            for ii in 0..parts.len() {
                sum_parts_now += parts[ii as usize];

                let x = map_range(
                    sum_parts_now,
                    0,
                    sum_parts_total,
                    app.window_rect().left(),
                    app.window_rect().right(),
                );
                let y = app.window_rect().top() - (row_height * i) as f32;
                let w = -map_range(parts[ii], 0, sum_parts_total, 0.0, app.window_rect().w());
                let h = row_height as f32;

                let index = counter % model.color_count;
                draw.rect().x_y(x + (w / 2.0), y - (h / 2.0)).w_h(w, h).hsv(
                    model.hue_values[index as usize],
                    model.saturation_values[index as usize],
                    model.brightness_values[index as usize],
                );

                counter += 1;
            }
        }
    }



}

fn mouse_released(app: &App, model: &mut Model, _button: MouseButton) {
    model.clicked = true;
    model.clicked_frame = app.elapsed_frames();
}

fn key_pressed(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
