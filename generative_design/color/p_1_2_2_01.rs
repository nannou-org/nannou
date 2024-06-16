// P_1_2_2_01
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
 * extract and sort the color palette of an image
 *
 * MOUSE
 * position x          : resolution
 *
 * KEYS
 * Q/W/E/R             : load different images
 * 1                   : no color sorting
 * 2                   : sort colors on hue
 * 3                   : sort colors on saturation
 * 4                   : sort colors on brightness
 * 5                   : sort colors on greyscale (luminance)
 * 6                   : sort colors on red
 * 7                   : sort colors on green
 * 8                   : sort colors on blue
 * 9                   : sort colors on alpha
 * s                   : save png
 * c                   : save color palette
 */
use nannou::image;
use nannou::image::GenericImageView;
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

enum SortMode {
    Red,
    Green,
    Blue,
    Hue,
    Saturation,
    Brightness,
    Grayscale,
    Alpha,
}

struct Model {
    image: image::DynamicImage,
    sort_mode: Option<SortMode>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(600, 600)
        .view(view)
        .key_released(key_released)
        .build();

    let assets = app.assets_path();
    let img_path = assets
        .join("images")
        .join("generative_examples")
        .join("pic1.jpg");

    let image = image::open(img_path).unwrap();
    Model {
        image,
        sort_mode: None,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(WHITE);
    let win = app.window_rect();
    let tile_count = clamp(
        map_range(app.mouse().x, win.left(), win.right(), 120, 1),
        120,
        1,
    );
    let rect_size = win.w() / tile_count as f32;

    let mut colors = Vec::new();

    for grid_y in 0..tile_count as usize {
        for grid_x in 0..tile_count as usize {
            let px = grid_x as f32 * rect_size + (rect_size / 2.0);
            let py = grid_y as f32 * rect_size + (rect_size / 2.0);
            // get current color
            let c = model.image.get_pixel(px as u32, py as u32);
            let red = c[0] as f32 / 255.0;
            let green = c[1] as f32 / 255.0;
            let blue = c[2] as f32 / 255.0;
            let alpha = c[3] as f32 / 255.0;

            colors.push(Srgba::new(red, green, blue, alpha));
        }
    }

    // sort
    if let Some(ref mode) = model.sort_mode {
        sort_colors(&mut colors, mode);
    }

    let mut i = 0;
    for grid_y in 0..tile_count as usize {
        for grid_x in 0..tile_count as usize {
            let pos_x = win.left() + grid_x as f32 * rect_size + (rect_size / 2.0);
            let pos_y = win.top() - grid_y as f32 * rect_size - (rect_size / 2.0);
            draw.rect()
                .x_y(pos_x, pos_y)
                .w_h(rect_size, rect_size)
                .color(colors[i]);
            i += 1;
        }
    }
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    let assets = app.assets_path();
    let img_path = assets.join("images").join("generative_examples");

    match key {
        KeyCode::KeyQ => {
            model.image = image::open(img_path.join("pic1.jpg")).unwrap();
        }
        KeyCode::KeyW => {
            model.image = image::open(img_path.join("pic2.jpg")).unwrap();
        }
        KeyCode::KeyE => {
            model.image = image::open(img_path.join("pic3.jpg")).unwrap();
        }
        KeyCode::KeyR => {
            model.image = image::open(img_path.join("pic4.jpg")).unwrap();
        }
        KeyCode::Digit1 => {
            model.sort_mode = None;
        }
        KeyCode::Digit2 => {
            model.sort_mode = Some(SortMode::Hue);
        }
        KeyCode::Digit3 => {
            model.sort_mode = Some(SortMode::Saturation);
        }
        KeyCode::Digit4 => {
            model.sort_mode = Some(SortMode::Brightness);
        }
        KeyCode::Digit5 => {
            model.sort_mode = Some(SortMode::Grayscale);
        }
        KeyCode::Digit6 => {
            model.sort_mode = Some(SortMode::Red);
        }
        KeyCode::Digit7 => {
            model.sort_mode = Some(SortMode::Green);
        }
        KeyCode::Digit8 => {
            model.sort_mode = Some(SortMode::Blue);
        }
        KeyCode::Digit9 => {
            model.sort_mode = Some(SortMode::Alpha);
        }
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        _otherkey => (),
    }
}

fn sort_colors(colors: &mut Vec<Srgba>, mode: &SortMode) {
    match mode {
        SortMode::Red => {
            colors.sort_by(|a, b| a.red.partial_cmp(&b.red).unwrap());
        }
        SortMode::Green => {
            colors.sort_by(|a, b| a.green.partial_cmp(&b.green).unwrap());
        }
        SortMode::Blue => {
            colors.sort_by(|a, b| a.blue.partial_cmp(&b.blue).unwrap());
        }
        SortMode::Hue => {
            colors.sort_by(|a, b| {
                let a: Hsla = a.clone().into();
                let b: Hsla = b.clone().into();
                a.hue.to_radians().partial_cmp(&b.hue.to_radians()).unwrap()
            });
        }
        SortMode::Saturation => {
            colors.sort_by(|a, b| {
                let a: Hsla = a.clone().into();
                let b: Hsla = b.clone().into();

                // temporary fix until conrod bug with saturation is resolved
                if a.saturation.is_nan() && b.saturation.is_nan() {
                    0.0.partial_cmp(&0.0).unwrap()
                } else if a.saturation.is_nan() {
                    0.0.partial_cmp(&b.saturation).unwrap()
                } else if b.saturation.is_nan() {
                    a.saturation.partial_cmp(&0.0).unwrap()
                } else {
                    a.saturation.partial_cmp(&b.saturation).unwrap()
                }
            });
        }
        SortMode::Brightness => {
            colors.sort_by(|a, b| {
                let a: Hsla = a.clone().into();
                let b: Hsla = b.clone().into();
                a.lightness.partial_cmp(&b.lightness).unwrap()
            });
        }
        SortMode::Grayscale => {
            colors.sort_by(|a, b| {
                let gray = |c: &Srgba| c.red * 0.222 + c.green * 0.707 + c.blue * 0.071;
                gray(a).partial_cmp(&gray(b)).unwrap()
            });
        }
        SortMode::Alpha => {
            colors.sort_by(|a, b| a.alpha.partial_cmp(&b.alpha).unwrap());
        }
    }
}
