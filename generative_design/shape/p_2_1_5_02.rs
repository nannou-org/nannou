// P_2_1_5_02
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
 * Place stacked circles of randomised heights at the mouse position
 * to create a moire effect drawing
 *
 * MOUSE
 * mouse              : place circle
 *
 * KEYS
 * s                   : save png
 *
 * CONTRIBUTED BY
 * [Niels Poldervaart](http://NielsPoldervaart.nl)
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

#[derive(Clone)]
struct Shape {
    x: f32,
    y: f32,
    r: f32,
}

impl Shape {
    fn new(x: f32, y: f32, r: f32) -> Self {
        Shape { x, y, r }
    }

    fn display(&self, draw: &Draw, model: &Model) {
        for i in (0..self.r as usize).step_by(model.density) {
            draw.ellipse()
                .x_y(self.x, self.y)
                .radius(i as f32 / 2.0)
                .resolution(200.0)
                .no_fill()
                .stroke_weight(1.25)
                .stroke(BLACK);
        }
    }
}

struct Model {
    shapes: Vec<Shape>,
    min_radius: f32,
    max_radius: f32,
    density: usize,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(800, 800)
        .view(view)
        .mouse_released(mouse_released)
        .key_released(key_released)
        .build();

    let shapes = vec![Shape::new(0.0, 0.0, app.window_rect().w()); 1];
    Model {
        shapes,
        min_radius: 5.0,
        max_radius: 250.0,
        density: 5,
    }
}

fn view(app: &App, model: &Model) {
    // Prepare to draw.
    let draw = app.draw();
    draw.background().color(WHITE);

    model.shapes.iter().for_each(|shape| {
        shape.display(&draw, model);
    });
}

fn mouse_released(app: &App, model: &mut Model, _button: MouseButton) {
    model.shapes.push(Shape::new(
        app.mouse().x,
        app.mouse().x,
        random_range(model.min_radius, model.max_radius),
    ));
}

fn key_released(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window()
            .save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
