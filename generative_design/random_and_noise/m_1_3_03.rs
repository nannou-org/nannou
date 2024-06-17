// M_1_3_03
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
 * creates a texture based on noise values
 *
 * MOUSE
 * position x/y        : specify noise input range
 *
 * KEYS
 * 1-2                 : set noise mode
 * arrow up            : noise falloff +
 * arrow down          : noise falloff -
 * arrow left          : noise octaves -
 * arrow right         : noise octaves +
 * s                   : save png
 */
use nannou::image;
use nannou::noise::{MultiFractal, NoiseFn, Seedable};
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    octaves: usize,
    falloff: f32,
    noise_mode: u8,
    noise_random_seed: u32,
    texture: Handle<Image>,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(512, 512)
        .view(view)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build();

    let window = app.main_window();
    let win = window.rect();
    let image = Image::new_fill(
        Extent3d {
            width: win.x().to_u32().unwrap(),
            height: win.y().to_u32().unwrap(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8Unorm,
        // Need to keep this image CPU persistent in order to add additional glyphs later on
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let texture = app.assets().add(image);
    Model {
        octaves: 4,
        falloff: 0.5,
        noise_mode: 1,
        noise_random_seed: 392,
        texture,
    }
}

fn view(app: &App, model: &Model) {
    let win = app.window_rect();
    let noise = nannou::noise::Fbm::new()
        .set_seed(model.noise_random_seed)
        .set_octaves(model.octaves)
        .set_persistence(model.falloff as f64);

    let noise_x_range = map_range(app.mouse().x, win.left(), win.right(), 0.0, win.w() / 10.0);
    let noise_y_range = map_range(app.mouse().x, win.top(), win.bottom(), 0.0, win.h() / 10.0);

    let image = image::ImageBuffer::from_fn(win.w() as u32, win.h() as u32, |x, y| {
        let noise_x = map_range(x, 0, win.w() as u32, 0.0, noise_x_range) as f64;
        let noise_y = map_range(y, 0, win.h() as u32, 0.0, noise_y_range) as f64;
        let mut noise_value = 0.0;

        if model.noise_mode == 1 {
            noise_value = map_range(
                noise.get([noise_x, noise_y]),
                1.0,
                -1.0,
                0.0,
                std::u8::MAX as f64,
            );
        } else if model.noise_mode == 2 {
            let n = map_range(
                noise.get([noise_x, noise_y]),
                -1.0,
                1.0,
                0.0,
                std::u8::MAX as f64 / 10.0,
            );
            noise_value = (n - n.floor()) * std::u8::MAX as f64;
        }
        let n = noise_value as u8;
        nannou::image::Rgba([n, n, n, std::u8::MAX])
    });

    let draw = app.draw();
    draw.rect().texture(&model.texture);
    draw.background().color(BLACK);
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        KeyCode::Space => {
            model.noise_random_seed = (random_f32() * 100000.0) as u32;
        }
        KeyCode::Digit1 => {
            model.noise_mode = 1;
        }
        KeyCode::Digit2 => {
            model.noise_mode = 2;
        }
        _otherkey => (),
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::ArrowUp => {
            model.falloff += 0.05;
        }
        KeyCode::ArrowDown => {
            model.falloff -= 0.05;
        }
        KeyCode::ArrowLeft => {
            model.octaves -= 1;
        }
        KeyCode::ArrowRight => {
            model.octaves += 1;
        }
        _otherkey => (),
    }

    if model.falloff > 1.0 {
        model.falloff = 1.0;
    }
    if model.falloff <= 0.0 {
        model.falloff = 0.0;
    }
    if model.octaves <= 1 {
        model.octaves = 1;
    }
}
