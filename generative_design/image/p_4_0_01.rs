// P_4_0_01
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
 * draw a grid of streched copies of an image
 *
 * MOUSE
 * position x           : tile count horizontally
 * position y           : tile count vertically
 *
 * KEYS
 * s                    : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    app.new_window()
        .size(650, 450)
        .view(view)
        .key_released(key_released)
        .build();
    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path();
    let img_path = assets
        .join("images")
        .join("generative_examples")
        .join("p_4_0_01.jpg");
    let texture = wgpu::Texture::from_path(app, img_path).unwrap();
    Model { texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    draw.background().color(BLACK);

    let draw = app.draw();
    let win = app.window_rect();
    let tile_count_x = map_range(app.mouse().x, win.left(), win.right(), 1.0, win.w() / 3.0);
    let tile_count_y = map_range(app.mouse().y, win.top(), win.bottom(), 1.0, win.h() / 3.0);
    let step_x = win.w() / tile_count_x;
    let step_y = win.h() / tile_count_y;

    for grid_y in (0..win.h() as usize).step_by(step_y as usize) {
        for grid_x in (0..win.w() as usize).step_by(step_x as usize) {
            let x = win.left() + grid_x as f32 + (step_x as f32 / 2.0);
            let y = win.top() - grid_y as f32 - (step_y as f32 / 2.0);
            draw.texture(&model.texture).x_y(x, y).w_h(step_x, step_y);
        }
    }


}

fn key_released(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
