// P_1_1_1_01
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
 * draw the color spectrum by moving the mouse
 *
 * MOUSE
 * position x/y        : resolution
 */
use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: Frame) {
    app.main_window().set_inner_size_pixels(800, 400);

    // Begin drawing
    let draw = app.draw();

    draw.background().color(BLACK);
    let win_rect = app.window_rect();

    let step_x = (app.mouse.x - win_rect.left()).max(5.0);
    let step_y = (win_rect.top() - app.mouse.y).max(5.0);

    let size = vec2(step_x, step_y);
    let r = nannou::geom::Rect::from_wh(size)
        .align_left_of(win_rect)
        .align_top_of(win_rect);
    let mut grid_y = 0.0;
    while grid_y < win_rect.h() {
        let mut grid_x = 0.0;
        while grid_x < win_rect.w() {
            let r = r.shift_x(grid_x).shift_y(-grid_y);
            let hue = grid_x / win_rect.w();
            let saturation = 1.0 - (grid_y / win_rect.h());
            draw.rect().xy(r.xy()).wh(r.wh()).hsl(hue, saturation, 0.5);
            grid_x += step_x;
        }
        grid_y += step_y;
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
