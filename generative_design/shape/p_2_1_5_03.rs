// P_2_1_5_03
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
 * Drawing tool for creating moire effect compositions using
 * rectangles of various widths, lengths, angles and colours.
 *
 * MOUSE
 * mouse               : place moire effect rectangle
 *
 * KEYS
 * 1                   : set color to red
 * 2                   : set color to green
 * 3                   : set color to blue
 * 4                   : set color to black
 * arrow up            : increase rectangle width
 * arrow down          : decrease rectangle width
 * s                   : save png
 *
 * CONTRIBUTED BY
 * [Niels Poldervaart](http://NielsPoldervaart.nl)
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone)]
struct Shape {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    h: f32,
    c: Vec3,
}

impl Shape {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32, h: f32, c: Vec3) -> Self {
        Shape {
            x1,
            y1,
            x2,
            y2,
            h,
            c,
        }
    }

    fn display(&self, draw: &Draw, model: &Model) {
        let w = pt2(self.x1, self.y1).distance(pt2(self.x2, self.y2));
        let a = (self.y2 - self.y1).atan2(self.x2 - self.x1);

        let draw = draw.x_y(self.x1, self.y1).rotate(a).x_y(0.0, -self.h / 2.0);

        for i in (0..self.h as usize).step_by(model.density) {
            draw.line()
                .start(pt2(0.0, i as f32))
                .end(pt2(w, i as f32))
                .rgb(self.c.x, self.c.y, self.c.z);
        }
    }
}

struct Model {
    shapes: Vec<Shape>,
    density: usize,
    shape_height: f32,
    shape_color: Vec3,
    new_shape: Option<Shape>,
    p_mouse: Point2,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(800, 800)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        shapes: Vec::new(),
        density: 3,
        shape_height: 64.0,
        shape_color: vec3(0.0, 0.0, 0.0),
        new_shape: None,
        p_mouse: vec2(0.0, 0.0),
    }
}

fn update(app: &App, model: &mut Model) {
    if let Some(ref mut s) = model.new_shape {
        s.x2 = app.mouse().x;
        s.y2 = app.mouse().x;
        s.h = model.shape_height;
        s.c = model.shape_color;
    }
}

fn view(app: &App, model: &Model) {
    // Prepare to draw.
    let draw = app.draw();
    draw.background().color(WHITE);

    model.shapes.iter().for_each(|shape| {
        shape.display(&draw, &model);
    });

    if let Some(ref s) = model.new_shape {
        s.display(&draw, &model);
    }

}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    model.p_mouse = app
        .mouse_buttons()
        .just_pressed(MouseButton::Left)
        .then_some(app.mouse())
        .unwrap();
    model.new_shape = Some(Shape::new(
        model.p_mouse.x,
        model.p_mouse.y,
        app.mouse().x,
        app.mouse().x,
        model.shape_height,
        model.shape_color,
    ));
}

fn mouse_released(_app: &App, model: &mut Model, button: MouseButton) {
    if let MouseButton::Left = button {
        if let Some(ref s) = model.new_shape {
            model.shapes.push(s.clone());
            model.new_shape = None;
        }
    }
}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyS => {
            app.main_window()
                .save_screenshot(app.exe_name().unwrap() + ".png");
        }
        KeyCode::Digit1 => {
            model.shape_color = vec3(1.0, 0.0, 0.0);
        }
        KeyCode::Digit2 => {
            model.shape_color = vec3(0.0, 1.0, 0.0);
        }
        KeyCode::Digit3 => {
            model.shape_color = vec3(0.0, 0.0, 1.0);
        }
        KeyCode::Digit4 => {
            model.shape_color = vec3(0.0, 0.0, 0.0);
        }
        KeyCode::ArrowUp => {
            model.shape_height += model.density as f32;
        }
        KeyCode::ArrowDown => {
            model.shape_height -= model.density as f32;
        }
        _other_key => (),
    }
}
