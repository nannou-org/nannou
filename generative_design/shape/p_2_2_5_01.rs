// P_2_2_5_01
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
 * pack as many circles as possible together
 *
 * MOUSE
 * press + position x/y : move area of interest
 *
 * KEYS
 * 1                    : show/hide circles
 * 2                    : show/hide lines
 * arrow up/down        : resize area of interest
 * f                    : freeze process. on/off
 * s                    : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone)]
struct Circle {
    x: f32,
    y: f32,
    r: f32,
}

impl Circle {
    fn new(x: f32, y: f32, r: f32) -> Self {
        Circle { x, y, r }
    }

    fn display(&self, draw: &Draw) {
        draw.ellipse()
            .x_y(self.x, self.y)
            .radius(self.r)
            .no_fill()
            .stroke(BLACK)
            .stroke_weight(1.5);
    }
}

struct Model {
    circles: Vec<Circle>,
    min_radius: usize,
    max_radius: usize,
    mouse_rect: f32,
    freeze: bool,
    show_circle: bool,
    show_line: bool,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    Model {
        circles: Vec::new(),
        min_radius: 3,
        max_radius: 50,
        mouse_rect: 15.0,
        freeze: false,
        show_circle: true,
        show_line: true,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let win = app.window_rect();

    // Choose a random or the current mouse position
    let mut new_x = random_range(
        win.left() + model.max_radius as f32,
        win.right() - model.max_radius as f32,
    );
    let mut new_y = random_range(
        win.top() - model.max_radius as f32,
        win.bottom() + model.max_radius as f32,
    );

    if app.mouse.buttons.left().is_down() {
        new_x = random_range(
            app.mouse.x - model.mouse_rect,
            app.mouse.x + model.mouse_rect,
        );
        new_y = random_range(
            app.mouse.y - model.mouse_rect,
            app.mouse.y + model.mouse_rect,
        );
    }

    // Try to fit the largest possible circle at the current location without overlapping
    let mut intersection = false;
    for new_r in (model.min_radius..=model.max_radius).rev() {
        for i in 0..model.circles.len() {
            let d = pt2(new_x, new_y).distance(pt2(model.circles[i].x, model.circles[i].y));
            intersection = d < model.circles[i].r + new_r as f32;
            if intersection {
                break;
            }
        }
        if !intersection {
            model.circles.push(Circle::new(new_x, new_y, new_r as f32));
            break;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    for i in 0..model.circles.len() {
        if model.show_line {
            // Try to find an adjacent circle to the current one and draw a connecting line between the two
            let mut closest_circle = None;
            for j in 0..model.circles.len() {
                let d = pt2(model.circles[i].x, model.circles[i].y)
                    .distance(pt2(model.circles[j].x, model.circles[j].y));
                if d <= model.circles[i].r + model.circles[j].r + 1.0 {
                    closest_circle = Some(model.circles[j].clone());
                    break;
                }
            }
            if closest_circle.is_some() {
                let closest = closest_circle.unwrap();
                draw.line()
                    .start(pt2(model.circles[i].x, model.circles[i].y))
                    .end(pt2(closest.x, closest.y))
                    .rgb(0.4, 0.9, 0.4);
            }
        }

        // Draw the circle itself
        if model.show_circle {
            model.circles[i].display(&draw);
        }
    }

    if app.mouse.buttons.left().is_down() {
        draw.rect()
            .x_y(app.mouse.x, app.mouse.y)
            .w_h(model.mouse_rect * 2.0, model.mouse_rect * 2.0)
            .no_fill()
            .stroke_weight(2.0)
            .stroke(rgb(0.4, 0.9, 0.4));
    }

    // Write to the window frame.
    draw.to_frame(app, &frame).unwrap();
}

fn key_released(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::S => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        Key::Up => {
            model.mouse_rect += 4.0;
        }
        Key::Down => {
            model.mouse_rect -= 4.0;
        }
        Key::F => {
            model.freeze = !model.freeze;
            if model.freeze {
                app.set_loop_mode(LoopMode::loop_once());
            } else {
                app.set_loop_mode(LoopMode::RefreshSync);
            }
        }
        Key::Key1 => {
            model.show_circle = !model.show_circle;
        }
        Key::Key2 => {
            model.show_line = !model.show_line;
        }
        _other_key => {}
    }
}
