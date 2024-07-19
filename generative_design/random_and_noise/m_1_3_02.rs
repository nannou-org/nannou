// M_1_3_02
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
 * creates a texture based on random values
 *
 * MOUSE
 * click               : new noise line
 *
 * KEYS
 * s                   : save png
 */
use nannou::image;
use nannou::image::DynamicImage;
use nannou::prelude::*;
use nannou::rand::rngs::SmallRng;
use nannou::rand::{Rng, SeedableRng};

fn main() {
    nannou::app(model).run();
}

struct Model {
    act_random_seed: u64,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(512, 512)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .key_pressed(key_pressed)
        .build();

    Model {
        act_random_seed: 42,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let win = app.window_rect();
    let mut rng = SmallRng::seed_from_u64(model.act_random_seed);

    let image: DynamicImage =
        image::ImageBuffer::from_fn(win.w() as u32, win.h() as u32, |_x, _y| {
            let r: u8 = rng.gen_range(0..std::u8::MAX);
            nannou::image::Rgba([r, r, r, std::u8::MAX])
        })
        .into();

    let image = Image::from_dynamic(image, true, default());
    let mut images = app.assets_mut::<Image>();
    let image = images.add(image);

    let draw = app.draw();
    draw.rect().w_h(win.w(), win.h()).texture(&image);
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.act_random_seed = (random_f32() * 100000.0) as u64;
}

fn key_pressed(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window()
            .save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
