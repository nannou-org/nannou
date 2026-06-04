// P_2_1_3_04
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
 * changing positions of stapled circles in a grid
 *
 * MOUSE
 * position x          : module detail
 * position y          : module parameter
 *
 * KEYS
 * 1-3                 : draw mode
 * arrow left/right    : number of tiles horizontally
 * arrow up/down       : number of tiles vertically
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    draw_mode: u8,
    tile_count_x: usize,
    tile_count_y: usize,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(900, 900)
        .view(view)
        .key_released(key_released)
        .build();

    Model {
        draw_mode: 1,
        tile_count_x: 6,
        tile_count_y: 6,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(WHITE);

    let win = app.window_rect();
    let count = map_range(app.mouse().x, win.left(), win.right(), 10, 60);
    let para = map_range(app.mouse().x, win.top(), win.bottom(), 0.0, 1.0);
    let tile_width = win.w() / model.tile_count_x as f32;
    let tile_height = win.h() / model.tile_count_y as f32;

    for grid_y in 0..=model.tile_count_y {
        for grid_x in 0..=model.tile_count_x {
            let pos_x = win.left() + (tile_width * grid_x as f32 + tile_width / 2.0);
            let pos_y = win.top() - (tile_height * grid_y as f32 + tile_height / 2.0);

            let mut draw = draw.x_y(pos_x, pos_y);

            match model.draw_mode {
                1 => {
                    let scale = 1.0 - 3.0 / count as f32;
                    for _ in 0..count {
                        draw.rect()
                            .x_y(0.0, 0.0)
                            .w_h(tile_width, tile_height)
                            .no_fill()
                            .stroke_weight(1.0 / scale)
                            .stroke(BLACK);
                        draw = draw.scale(scale).rotate(para * 0.1);
                    }
                }
                2 => {
                    for i in 0..count {
                        let gradient =
                            Vec3::ZERO.lerp(vec3(0.14, 1.0, 0.71), i as f32 / count as f32);
                        draw = draw.rotate(PI / 4.0);
                        draw.rect().x_y(0.0, 0.0).w_h(tile_width, tile_height).hsla(
                            gradient.x,
                            gradient.y,
                            gradient.z,
                            i as f32 / count as f32,
                        );
                        draw = draw.scale(1.0 - 3.0 / count as f32).rotate(para * 1.5);
                    }
                }
                3 => {
                    for i in 0..count {
                        let gradient =
                            vec3(0.0, 0.5, 0.64).lerp(vec3(1.0, 1.0, 1.0), i as f32 / count as f32);
                        let draw2 = draw.x_y(4.0 * i as f32, 0.0);
                        draw2
                            .ellipse()
                            .x_y(0.0, 0.0)
                            .w_h(tile_width / 4.0, tile_height / 4.0)
                            .resolution(12.0)
                            .srgba(gradient.x, gradient.y, gradient.z, 0.66);

                        let draw3 = draw.x_y(-4.0 * i as f32, 0.0);
                        draw3
                            .ellipse()
                            .x_y(0.0, 0.0)
                            .w_h(tile_width / 4.0, tile_height / 4.0)
                            .resolution(12.0)
                            .srgba(gradient.x, gradient.y, gradient.z, 0.66);

                        draw = draw.scale(1.0 - 1.5 / count as f32).rotate(para * 1.5);
                    }
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
        KeyCode::ArrowDown => {
            model.tile_count_y = (model.tile_count_y - 1).max(1);
        }
        KeyCode::ArrowUp => {
            model.tile_count_y += 1;
        }
        KeyCode::ArrowLeft => {
            model.tile_count_x = (model.tile_count_x - 1).max(1);
        }
        KeyCode::ArrowRight => {
            model.tile_count_x += 1;
        }
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
