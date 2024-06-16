// P_4_3_1_01
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
 * pixel mapping. each pixel is translated into a new element
 *
 * MOUSE
 * position x/y        : various parameters (depending on draw mode)
 *
 * KEYS
 * 1-9                 : switch draw mode
 * s                   : save png
 */
use nannou::prelude::*;

use nannou::image;
use nannou::image::GenericImageView;

fn main() {
    nannou::app(model).run();
}

struct Model {
    image: image::DynamicImage,
    draw_mode: u8,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    app.new_window()
        .size(603, 873)
        .view(view)
        .key_released(key_released)
        .build();

    let assets = app.assets_path();
    let img_path = assets
        .join("images")
        .join("generative_examples")
        .join("p_4_3_1_01.png");

    let image = image::open(img_path).unwrap();
    Model {
        image,
        draw_mode: 1,
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(WHITE);
    let win = app.window_rect();

    let mouse_x_factor = map_range(app.mouse().x, win.left(), win.right(), 0.01, 1.0);
    let mouse_y_factor = map_range(app.mouse().x, win.bottom(), win.top(), 0.01, 1.0);

    let (w, h) = model.image.dimensions();
    for grid_x in 0..w {
        for grid_y in 0..h {
            // get current color
            let c = model.image.get_pixel(grid_x, grid_y);
            // greyscale conversion
            let red = c[0] as f32 / 255.0;
            let green = c[1] as f32 / 255.0;
            let blue = c[2] as f32 / 255.0;
            let greyscale = red * 0.222 + green * 0.707 + blue * 0.071;

            // Grid position + tile size
            let tile_width = win.w() / w as f32;
            let tile_height = win.h() / h as f32;
            let pos_x = win.left() + tile_width * grid_x as f32 + (tile_width / 2.0);
            let pos_y = win.top() - tile_height * grid_y as f32 - (tile_height / 2.0);

            match model.draw_mode {
                1 => {
                    // greyscale to stroke weight
                    let w1 = map_range(greyscale, 0.0, 1.0, 15.0, 0.1);
                    draw.line()
                        .start(pt2(pos_x, pos_y))
                        .end(pt2(pos_x + 5.0, pos_y + 5.0))
                        .weight(w1 * mouse_x_factor)
                        .caps_round()
                        .color(BLACK);
                }
                2 => {
                    // greyscale to ellise area
                    let mut r2 = 1.1284 * (tile_width * tile_width * (1.0 - greyscale)).sqrt();
                    r2 *= mouse_x_factor * 3.0;
                    draw.ellipse()
                        .x_y(pos_x, pos_y)
                        .radius(r2 / 2.0)
                        .color(BLACK);
                }
                3 => {
                    // greyscale to line length
                    let mut l3 = map_range(greyscale, 0.0, 1.0, 30.0, 0.1);
                    l3 *= mouse_x_factor;
                    draw.line()
                        .start(pt2(pos_x, pos_y))
                        .end(pt2(pos_x + l3, pos_y - l3))
                        .weight(10.0 * mouse_y_factor)
                        .caps_round()
                        .color(BLACK);
                }
                4 => {
                    // greyscale to rotation, line length and stroke weight
                    let w4 = map_range(greyscale, 0.0, 1.0, 10.0, 0.0);
                    let mut l4 = map_range(greyscale, 0.0, 1.0, 35.0, 0.0);
                    l4 *= mouse_x_factor;

                    let draw = draw.x_y(pos_x, pos_y).rotate(greyscale * PI);
                    draw.line()
                        .start(pt2(0.0, 0.0))
                        .end(pt2(l4, -l4))
                        .weight(w4 * mouse_x_factor + 0.1)
                        .caps_round()
                        .color(BLACK);
                }
                5 => {
                    // greyscale to line relief
                    let w5 = map_range(greyscale, 0.0, 1.0, 5.0, 0.2);
                    // get neighbour pixel, limit it to image width
                    let c2 = model.image.get_pixel((grid_x + 1).min(w - 1), grid_y);
                    // greyscale conversion
                    let red = c2[0] as f32 / 255.0;
                    let green = c2[1] as f32 / 255.0;
                    let blue = c2[2] as f32 / 255.0;
                    let greyscale2 = red * 0.222 + green * 0.707 + blue * 0.071;
                    let h5 = 50.0 * mouse_x_factor;
                    let d1 = map_range(greyscale, 0.0, 1.0, h5, 0.0);
                    let d2 = map_range(greyscale2, 0.0, 1.0, h5, 0.0);

                    draw.line()
                        .start(pt2(pos_x - d1, pos_y - d1))
                        .end(pt2(pos_x + tile_width - d2, pos_y - d2))
                        .weight(w5 * mouse_y_factor + 0.1)
                        .rgb(red, green, blue);
                }
                6 => {
                    // pixel color to fill, greyscale to ellipse size
                    let w6 = map_range(greyscale, 0.0, 1.0, 25.0, 0.0);
                    draw.ellipse()
                        .x_y(pos_x, pos_y)
                        .w_h(w6 * mouse_x_factor, w6 * mouse_x_factor)
                        .rgb(red, green, blue);
                }
                7 => {
                    let w7 = map_range(greyscale, 0.0, 1.0, 5.0, 0.1);
                    let draw = draw
                        .x_y(pos_x, pos_y)
                        .rotate(greyscale * PI * mouse_y_factor);
                    draw.rect()
                        .x_y(0.0, 0.0)
                        .w_h(15.0, 15.0)
                        .stroke_weight(w7)
                        .stroke(Color::srgb(red, green, blue))
                        .rgba(1.0, 1.0, 1.0, mouse_x_factor);
                }
                8 => {
                    let col = Color::srgb(greyscale, greyscale * mouse_x_factor, mouse_y_factor);
                    draw.rect().x_y(pos_x, pos_y).w_h(3.5, 3.5).color(col);
                    draw.rect().x_y(pos_x + 4.0, pos_y).w_h(3.5, 3.5).color(col);
                    draw.rect().x_y(pos_x, pos_y - 4.0).w_h(3.5, 3.5).color(col);
                    draw.rect()
                        .x_y(pos_x + 4.0, pos_y - 4.0)
                        .w_h(3.5, 3.5)
                        .color(col);
                }
                9 => {
                    let draw = draw.x_y(pos_x, pos_y).rotate(greyscale * PI);
                    draw.rect()
                        .x_y(0.0, 0.0)
                        .w_h(15.0 * mouse_x_factor, 15.0 * mouse_y_factor)
                        .stroke_weight(1.0)
                        .stroke(Color::srgb(1.0, greyscale, 0.0))
                        .no_fill();
                    let w9 = map_range(greyscale, 0.0, 1.0, 15.0, 0.1);
                    draw.ellipse()
                        .x_y(0.0, 0.0)
                        .w_h(5.0, 2.5)
                        .stroke_weight(w9)
                        .stroke(Color::srgb(0.0, 0.0, 0.27))
                        .no_fill();
                }
                _ => (),
            }
        }
    }
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::Digit1 => {
            model.draw_mode = 1;
        }
        KeyCode::Digit2 => {
            model.draw_mode = 2;
        }
        KeyCode::Digit3 => {
            model.draw_mode = 3;
        }
        KeyCode::Digit4 => {
            model.draw_mode = 4;
        }
        KeyCode::Digit5 => {
            model.draw_mode = 5;
        }
        KeyCode::Digit6 => {
            model.draw_mode = 6;
        }
        KeyCode::Digit7 => {
            model.draw_mode = 7;
        }
        KeyCode::Digit8 => {
            model.draw_mode = 8;
        }
        KeyCode::Digit9 => {
            model.draw_mode = 9;
        }
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        _otherkey => (),
    }
}
