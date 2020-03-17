// M_5_1_01
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
 * simple example of a recursive function
 *
 * KEYS
 * 1-9                 : recursion level
 * s                   : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    recursion_level: u8,
    start_radius: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();
    Model {
        recursion_level: 6,
        start_radius: 200.0,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    draw_branch(&draw, 0.0, 0.0, model.start_radius, model.recursion_level);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

// Recursive function
fn draw_branch(draw: &app::Draw, x: f32, y: f32, radius: f32, level: u8) {
    use nannou::geom::path::Builder;
    let mut builder = Builder::new();
    builder = builder.arc(pt2(x, y), vec2(radius / 2.0, radius / 2.0), -PI, 0.0);
    let arc_path = builder.build();

    // draw arc
    draw.path()
        .stroke()
        .stroke_weight(level as f32 * 2.0)
        .rgba(0.0, 0.5, 0.64, 0.4)
        .events(arc_path.iter());

    // draw center dot
    draw.ellipse()
        .x_y(x, y)
        .radius(level as f32 * 0.75)
        .color(BLACK);

    // as long as level is greater than zero, draw sub-branches
    if level > 0 {
        // left branch
        draw_branch(&draw, x - radius, y - radius / 2.0, radius / 2.0, level - 1);
        // right branch
        draw_branch(&draw, x + radius, y - radius / 2.0, radius / 2.0, level - 1);
    }
}

fn key_released(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => model.recursion_level = 1,
        Key::Key2 => model.recursion_level = 2,
        Key::Key3 => model.recursion_level = 3,
        Key::Key4 => model.recursion_level = 4,
        Key::Key5 => model.recursion_level = 5,
        Key::Key6 => model.recursion_level = 6,
        Key::Key7 => model.recursion_level = 7,
        Key::Key8 => model.recursion_level = 8,
        Key::Key9 => model.recursion_level = 9,
        Key::Key0 => model.recursion_level = 0,
        Key::S => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
