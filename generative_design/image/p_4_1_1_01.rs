// P_4_1_1_01
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
 * cutting and multiplying an area of the image
 *
 * MOUSE
 * position x/y         : area position
 * left click           : multiply the area
 *
 * KEYS
 * 1-3                  : area size
 * r                    : toggle random area
 * s                    : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    texture: Handle<Image>,
    tile_count_x: usize,
    tile_count_y: usize,
    tile_count: usize,
    img_tiles: Vec<Rect>,
    tile_width: f32,
    tile_height: f32,
    crop_x: f32,
    crop_y: f32,
    select_mode: bool,
    random_mode: bool,
    selected_mouse_pos: Point2,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    app.new_window()
        .size(1600, 1200)
        .view(view)
        .mouse_moved(mouse_moved)
        .mouse_released(mouse_released)
        .key_released(key_released)
        .build();
    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path();
    let img_path = assets
        .join("images")
        .join("generative_examples")
        .join("p_4_1_1_01.jpg");
    let texture = app.assets().load(img_path);

    let win = app.window_rect();
    let tile_count_x = 4;
    let tile_count_y = 4;
    Model {
        texture,
        tile_count_x,
        tile_count_y,
        tile_count: tile_count_x * tile_count_y,
        img_tiles: Vec::new(),
        tile_width: win.w() / tile_count_x as f32,
        tile_height: win.h() / tile_count_y as f32,
        crop_x: 100.0,
        crop_y: 100.0,
        select_mode: true,
        random_mode: false,
        selected_mouse_pos: pt2(0.0, 0.0),
    }
}

fn crop_tiles(app: &App, model: &mut Model, win: Rect) {
    model.tile_width = win.w() / model.tile_count_y as f32;
    model.tile_height = win.h() / model.tile_count_x as f32;
    model.tile_count = model.tile_count_x * model.tile_count_y;
    model.img_tiles = Vec::new();
    for _ in 0..model.tile_count_y {
        for _ in 0..model.tile_count_x {
            if model.random_mode {
                model.crop_x = random_range(
                    app.mouse().x - model.tile_width / 2.0,
                    app.mouse().x + model.tile_width / 2.0,
                );
                model.crop_y = random_range(
                    app.mouse().x - model.tile_height / 2.0,
                    app.mouse().x + model.tile_height / 2.0,
                );
            }
            model.crop_x = clamp(
                model.crop_x,
                win.left() + (model.tile_width as f32 / 2.0),
                win.right() - (model.tile_width as f32 / 2.0),
            );
            model.crop_y = clamp(
                model.crop_y,
                win.top() - (model.tile_height as f32 / 2.0),
                win.bottom() + (model.tile_height as f32 / 2.0),
            );

            let [w, h] = model.texture.size();
            let area = geom::Rect::from_x_y_w_h(
                map_range(model.crop_x, win.left(), win.right(), 0.0, 1.0),
                map_range(model.crop_y, win.top(), win.bottom(), 0.0, 1.0),
                model.tile_width / w as f32,
                model.tile_height / h as f32,
            );
            model.img_tiles.push(area);
        }
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let win = app.window_rect();

    draw.background().color(BLACK);

    if model.select_mode {
        // in selection mode, a white selection rectangle is drawn over the image
        draw.texture(&model.texture);
        draw.rect()
            .x_y(model.crop_x, model.crop_y)
            .w_h(model.tile_width, model.tile_height)
            .no_fill()
            .stroke_weight(2.0)
            .stroke(WHITE);
    } else {
        // reassemble image
        let mut index = 0;
        for grid_y in 0..model.tile_count_y {
            for grid_x in 0..model.tile_count_x {
                let x =
                    win.left() + grid_x as f32 * model.tile_width + (model.tile_width as f32 / 2.0);
                let y = win.top()
                    - grid_y as f32 * model.tile_height
                    - (model.tile_height as f32 / 2.0);
                draw.texture(&model.texture)
                    .x_y(x, y)
                    .w_h(model.tile_width, model.tile_height)
                    .area(model.img_tiles[index]);
                index += 1;
            }
        }
    }
}

fn mouse_released(app: &App, model: &mut Model, _button: MouseButton) {
    model.select_mode = false;
    crop_tiles(app, model, app.window_rect());
}

fn mouse_moved(app: &App, model: &mut Model, pos: Point2) {
    let win = app.window_rect();

    if pos != model.selected_mouse_pos {
        model.select_mode = true;
        model.selected_mouse_pos = pos;
    }

    let htw = model.tile_width / 2.0;
    let hth = model.tile_height / 2.0;
    model.crop_x = clamp(app.mouse().x, win.left() + htw, win.right() - htw);
    model.crop_y = clamp(app.mouse().x, win.top() - hth, win.bottom() + hth);
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::Digit1 => {
            model.tile_count_x = 4;
            model.tile_count_y = 4;
            crop_tiles(app, model, app.window_rect());
        }
        KeyCode::Digit2 => {
            model.tile_count_x = 10;
            model.tile_count_y = 10;
            crop_tiles(app, model, app.window_rect());
        }
        KeyCode::Digit3 => {
            model.tile_count_x = 20;
            model.tile_count_y = 20;
            crop_tiles(app, model, app.window_rect());
        }
        KeyCode::KeyR => {
            model.random_mode = !model.random_mode;
        }
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
