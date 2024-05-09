// P_2_2_4_02
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
 * limited diffusion aggregation
 *
 * KEYS
 * 1             : toggle draw original position of circles
 * s             : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    x: Vec<f32>,
    y: Vec<f32>,
    r: Vec<f32>,
    x2: Vec<f32>,
    y2: Vec<f32>,
    draw_ghosts: bool,
    current_count: usize,
    max_count: usize,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    let max_count = 5000; // max count of the circles
    Model {
        x: vec![0.0; max_count],
        y: vec![0.0; max_count],
        r: vec![360.0; max_count],
        x2: vec![0.0; max_count],
        y2: vec![0.0; max_count],
        draw_ghosts: false,
        current_count: 1,
        max_count,
    }
}

fn update(app: &App, model: &mut Model) {
    let win = app.window_rect();

    // create a random set of parameters
    let new_r = random_range(1.0, 7.0);
    let new_x = random_range(win.left() + new_r, win.right() - new_r);
    let new_y = random_range(win.top() - new_r, win.bottom() + new_r);
    let mut closest_dist = std::f32::MAX;
    let mut closest_index = 0;
    // which circle is the closest?
    for i in 0..model.current_count {
        let new_dist = pt2(new_x, new_y).distance(pt2(model.x[i], model.y[i]));
        if new_dist < closest_dist {
            closest_dist = new_dist;
            closest_index = i;
        }
    }

    // aline it to the closest circle outline
    let angle = (new_y - model.y[closest_index]).atan2(new_x - model.x[closest_index]);

    model.x2[model.current_count] = new_x;
    model.y2[model.current_count] = new_y;
    model.x[model.current_count] =
        model.x[closest_index] + angle.cos() * (model.r[closest_index] + new_r);
    model.y[model.current_count] =
        model.y[closest_index] + angle.sin() * (model.r[closest_index] + new_r);
    model.r[model.current_count] = new_r;
    model.current_count += 1;

    if model.current_count >= model.max_count {
        app.set_loop_mode(LoopMode::loop_once());
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(WHITE);

    // draw circles at random position and lines
    if model.draw_ghosts {
        for i in 1..model.current_count {
            draw.ellipse()
                .x_y(model.x2[i], model.y2[i])
                .radius(model.r[i])
                .gray(0.9)
                .stroke(BLACK);
            draw.line()
                .start(pt2(model.x2[i], model.y2[i]))
                .end(pt2(model.x[i], model.y[i]))
                .color(BLACK);
        }
    }

    for i in 0..model.current_count {
        if i == 0 {
            draw.ellipse()
                .x_y(model.x[i], model.y[i])
                .radius(model.r[i])
                .no_fill()
                .stroke(BLACK);
        } else {
            draw.ellipse()
                .x_y(model.x[i], model.y[i])
                .radius(model.r[i])
                .gray(0.2);
        }
    }

    // Write to the window frame.

}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
    if key == KeyCode::Digit1 {
        model.draw_ghosts = !model.draw_ghosts;
    }
}
