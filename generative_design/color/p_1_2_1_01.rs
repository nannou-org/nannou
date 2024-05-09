// P_1_2_1_01
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
 * shows how to interpolate colors in different styles/ color modes
 *
 * MOUSE
 * left click          : new random color set
 * position x          : interpolation resolution
 * position y          : row count
 *
 * KEYS
 * 1-2                 : switch interpolation style
 * s                   : save png
 * c                   : save color palette
 */
use nannou::prelude::*;


fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    tile_count_x: usize,
    tile_count_y: usize,
    colours_left: Vec<Hsva>,
    colours_right: Vec<Hsva>,
    interpolate_shortest: bool,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .key_pressed(key_pressed)
        .mouse_released(mouse_released)
        .view(view)
        .build()
        .unwrap();

    let tile_count_x = 2;
    let tile_count_y = 10;
    let mut model = Model {
        tile_count_x,
        tile_count_y,
        colours_left: vec![Color::hsv(0.0, 0.0, 0.0); tile_count_y],
        colours_right: vec![Color::hsv(0.0, 0.0, 0.0); tile_count_y],
        interpolate_shortest: true,
    };
    shake_colors(&mut model);

    model
}

fn update(app: &App, model: &mut Model) {
    let win = app.window_rect();
    model.tile_count_x = clamp(
        map_range(app.mouse().x, win.left(), win.right(), 2, 100),
        2,
        100,
    ) as usize;
    model.tile_count_y = clamp(
        map_range(app.mouse().y, win.top(), win.bottom(), 2, 10),
        2,
        10,
    ) as usize;
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(BLACK);
    let win = app.window_rect();

    let tile_width = win.w() / model.tile_count_x as f32;
    let tile_height = win.h() / model.tile_count_y as f32;

    for grid_y in 0..model.tile_count_y {
        let col1 = model.colours_left[grid_y];
        let col2 = model.colours_right[grid_y];

        for grid_x in 0..model.tile_count_x {
            let amount = map_range(grid_x, 0, model.tile_count_x - 1, 0.0, 1.0);
            let pos_x = win.left() + tile_width * grid_x as f32;
            let pos_y = win.top() - tile_height * grid_y as f32;

            let col = if model.interpolate_shortest {
                let c1 = cast_to_rgb(col1);
                let c2 = cast_to_rgb(col2);
                Hsva::from_rgb(c1.mix(&c2, amount))
            } else {
                col1.mix(&col2, amount)
            };
            draw.rect()
                .x_y(pos_x + (tile_width / 2.0), pos_y - (tile_height / 2.0))
                .w_h(tile_width, tile_height)
                .color(col);
        }
    }



}

fn cast_to_rgb(col: Hsva) -> LinearRgba {
    let red: f32 = col.hue.into();
    let green = col.saturation;
    let blue = col.value;
    LinearRgba::new(red / 360.0, green, blue, 1.0)
}

fn shake_colors(model: &mut Model) {
    for i in 0..model.tile_count_y {
        model.colours_left[i] = Color::hsv(random_f32() * 0.166, random_f32(), 1.0);
        model.colours_right[i] = Color::hsv(random_range(0.44, 0.52), 1.0, random_f32());
    }
}

fn mouse_released(_app: &App, model: &mut Model, _button: MouseButton) {
    shake_colors(model);
}

fn key_pressed(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::Digit1 => {
            model.interpolate_shortest = true;
        }
        KeyCode::Digit2 => {
            model.interpolate_shortest = false;
        }
        KeyCode::KeyS => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
