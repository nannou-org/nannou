// P_1_2_3_01
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
 * generates specific color palettes
 *
 * MOUSE
 * position x/y        : row and coloum count
 *
 * KEYS
 * 0-9                 : creates specific color palettes
 * s                   : save png
 * c                   : save color palette
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    tile_count_x: usize,
    tile_count_y: usize,
    hue_values: Vec<f32>,
    saturation_values: Vec<f32>,
    brightness_values: Vec<f32>,
}

fn model(app: &App) -> Model {
    let tile_count_x = 50;
    let tile_count_y = 10;

    let hue_values = (0..tile_count_x).map(|_| random()).collect();
    let saturation_values = (0..tile_count_x).map(|_| random()).collect();
    let brightness_values = (0..tile_count_x).map(|_| random()).collect();

    app.new_window()
        .size(720, 720)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    Model {
        tile_count_x,
        tile_count_y,
        hue_values,
        saturation_values,
        brightness_values,
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = random();
                model.saturation_values[i] = random();
                model.brightness_values[i] = random();
            }
        }
        Key::Key2 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = random();
                model.saturation_values[i] = random();
                model.brightness_values[i] = 1.0;
            }
        }
        Key::Key3 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = random();
                model.saturation_values[i] = 1.0;
                model.brightness_values[i] = random();
            }
        }
        Key::Key4 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = 0.0;
                model.saturation_values[i] = 0.0;
                model.brightness_values[i] = random();
            }
        }
        Key::Key5 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = 0.54;
                model.saturation_values[i] = 1.0;
                model.brightness_values[i] = random();
            }
        }
        Key::Key6 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = 0.54;
                model.saturation_values[i] = random();
                model.brightness_values[i] = 1.0;
            }
        }
        Key::Key7 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = random_f32() * 0.5;
                model.saturation_values[i] = random_f32() * 0.2 + 0.8;
                model.brightness_values[i] = random_f32() * 0.4 + 0.5;
            }
        }
        Key::Key8 => {
            for i in 0..model.tile_count_x {
                model.hue_values[i] = random_f32() * 0.5 + 0.5;
                model.saturation_values[i] = random_f32() * 0.2 + 0.8;
                model.brightness_values[i] = random_f32() * 0.4 + 0.5;
            }
        }
        Key::Key9 => {
            for i in 0..model.tile_count_x {
                if i % 2 == 0 {
                    model.hue_values[i] = random();
                    model.saturation_values[i] = 1.0;
                    model.brightness_values[i] = random();
                } else {
                    model.hue_values[i] = 0.54;
                    model.saturation_values[i] = random();
                    model.brightness_values[i] = 1.0;
                }
            }
        }
        Key::Key0 => {
            for i in 0..model.tile_count_x {
                if i % 2 == 0 {
                    model.hue_values[i] = 0.38;
                    model.saturation_values[i] = random_f32() * 0.7 + 0.3;
                    model.brightness_values[i] = random_f32() * 0.6 + 0.4;
                } else {
                    model.hue_values[i] = 0.58;
                    model.saturation_values[i] = random_f32() * 0.6 + 0.4;
                    model.brightness_values[i] = random_f32() * 0.5 + 0.5;
                }
            }
        }
        _other_key => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    // white black
    draw.background().rgb(0.0, 0.0, 0.2);
    let win_rect = app.window_rect();

    // limit mouse coordintes to canvas
    let mx = (app.mouse.x - win_rect.left()).max(1.0).min(win_rect.w());
    let my = (win_rect.top() - app.mouse.y).max(1.0).min(win_rect.h());

    // tile counter
    let mut counter = 0;

    // map mouse to grid resolution
    let current_tile_count_x =
        map_range(mx, 0.0, win_rect.w(), 1.0, model.tile_count_x as f32) as i32;
    let current_tile_count_y =
        map_range(my, 0.0, win_rect.h(), 1.0, model.tile_count_y as f32) as i32;
    let tile_width = win_rect.w() as i32 / current_tile_count_x;
    let tile_height = win_rect.h() as i32 / current_tile_count_y;

    let size = vec2(tile_width as f32, tile_height as f32);
    let r = nannou::geom::Rect::from_wh(size)
        .align_left_of(win_rect)
        .align_top_of(win_rect);
    let mut grid_y = 0;
    while grid_y < model.tile_count_y {
        let mut grid_x = 0;
        while grid_x < model.tile_count_x {
            let r = r
                .shift_x((tile_width * grid_x as i32) as f32)
                .shift_y(-(tile_height * grid_y as i32) as f32);
            let index = counter % current_tile_count_x as usize;
            draw.rect().xy(r.xy()).wh(r.wh()).hsv(
                model.hue_values[index],
                model.saturation_values[index],
                model.brightness_values[index],
            );
            counter += 1;
            grid_x += 1;
        }
        grid_y += 1;
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
