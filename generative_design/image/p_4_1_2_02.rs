// P_4_1_2_02
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
 * image feedback process.
 *
 * KEYS
 * del, backspace      : clear screen
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    texture: Handle<Image>,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    app.new_window()
        .size(1024, 580)
        .view(view)
        .key_released(key_released)
        .build();

    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path();
    let img_path = assets
        .join("images")
        .join("generative_examples")
        .join("p_4_1_2_01.png");
    let texture = app.assets().load(img_path);
    Model { texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let Some(texture) = app.image(&model.texture) else {
        return;
    };

    let win = app.window_rect();

    let w = 150.0;
    let h = 50.0;
    let x1 = random_range(win.left(), win.right());
    let y1 = random_range(win.bottom(), win.top());
    let x2 = x1 + random_range(-10.0, 10.0);
    let y2 = y1 - random_range(-10.0, 10.0);

    let area = geom::Rect::from_x_y_w_h(
        map_range(x1, win.left(), win.right(), 0.0, 1.0),
        map_range(y1, win.top(), win.bottom(), 0.0, 1.0),
        map_range(w, 0.0, win.w(), 0.0, 1.0),
        map_range(h, 0.0, win.h(), 0.0, 1.0),
    );

    // Don't interpolate between pixels.model

    let dim = texture.dimensions();
    let texture = model.texture.clone();
    if app.elapsed_frames() == 0 || app.keys().just_pressed(KeyCode::Delete) {
        draw.background().color(WHITE);
        draw.texture(texture, dim);
    } else {
        draw.texture(texture, dim).x_y(x2, y2).w_h(w, h).area(area);
    }
}

fn key_released(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window()
            .save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
