// P_2_1_3_02
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
 * draw a module made of lines in a grid
 *
 * MOUSE
 * position x          : number of tiles horizontally
 * position y          : number of tiles vertically
 *
 * KEYS
 * 1-3                 : draw mode
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    background_color: f32,
    draw_mode: u8,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(600, 600)
        .view(view)
        .key_released(key_released)
        .build();

    Model {
        background_color: 0.0,
        draw_mode: 1,
    }
}

fn update(_app: &App, model: &mut Model) {
    match model.draw_mode {
        1 => model.background_color = 1.0,
        2 => model.background_color = 1.0,
        3 => model.background_color = 0.0,
        _ => unreachable!(),
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let g = model.background_color;
    draw.background().rgb(g, g, g);

    let win = app.window_rect();
    let count = 10;

    let tile_count_x = map_range(app.mouse().x, win.left(), win.right(), 1, 30);
    let tile_count_y = map_range(app.mouse().x, win.top(), win.bottom(), 1, 30);
    let tile_width = win.w() / tile_count_x as f32;
    let tile_height = win.h() / tile_count_y as f32;

    let mut line_weight = 0.0;
    let mut stroke_color = 0.0;

    for grid_y in 0..=tile_count_y {
        for grid_x in 0..=tile_count_x {
            let pos_x = win.left() + (tile_width * grid_x as f32);
            let pos_y = win.top() - (tile_height * grid_y as f32);

            let x1 = tile_width / 2.0;
            let y1 = tile_height / 2.0;
            let mut x2 = 0.0;
            let mut y2 = 0.0;

            let draw = draw.x_y(pos_x, pos_y);

            for side in 0..4 {
                for i in 0..count {
                    // move end point around the four sides of the tile
                    match side {
                        0 => {
                            x2 += tile_width / count as f32;
                            y2 = 0.0;
                        }
                        1 => {
                            x2 = tile_width;
                            y2 += tile_height / count as f32;
                        }
                        2 => {
                            x2 -= tile_width / count as f32;
                            y2 = tile_height;
                        }
                        3 => {
                            x2 = 0.0;
                            y2 -= tile_height / count as f32;
                        }
                        _ => unreachable!(),
                    }

                    // adjust weight and color of the line
                    if i < count / 2 {
                        line_weight += 1.0;
                        stroke_color += 0.25;
                    } else {
                        line_weight -= 1.0;
                        stroke_color -= 0.25;
                    }

                    // set colors depending on draw mode
                    match model.draw_mode {
                        1 => {
                            stroke_color = 0.0;
                            line_weight = 1.0;
                        }
                        2 => {
                            stroke_color = 0.0;
                        }
                        3 => {
                            line_weight = map_range(app.mouse().x, win.left(), win.right(), 1.0, 8.0);
                        }
                        _ => unreachable!(),
                    }

                    // draw the line
                    draw.line()
                        .start(pt2(x1, y1))
                        .end(pt2(x2, y2))
                        .weight(line_weight)
                        .caps_round()
                        .gray(stroke_color);
                }
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
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
