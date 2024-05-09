// P_2_2_3_01
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
 * form morphing process by connected random agents
 *
 * MOUSE
 * click               : start a new circe
 * position x/y        : direction of floating
 *
 * KEYS
 * 1-2                 : fill styles
 * f                   : freeze. loop on/off
 * Delete/Backspace    : clear display
 * s                   : save png
 */
use nannou::lyon;
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    form_resolution: usize,
    step_size: f32,
    init_radius: f32,
    center_x: f32,
    center_y: f32,
    x: Vec<f32>,
    y: Vec<f32>,
    filled: bool,
    freeze: bool,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .key_released(key_released)
        .mouse_pressed(mouse_pressed)
        .build()
        .unwrap();

    let form_resolution = 150;
    let angle = (360.0 / form_resolution as f32).to_radians();
    let init_radius = 150.0;
    let mut x = Vec::new();
    let mut y = Vec::new();
    for i in 0..form_resolution {
        x.push((angle * i as f32).cos() * init_radius);
        y.push((angle * i as f32).sin() * init_radius);
    }
    Model {
        form_resolution,
        step_size: 2.0,
        init_radius,
        center_x: 0.0,
        center_y: 0.0,
        x,
        y,
        filled: false,
        freeze: false,
    }
}

fn update(app: &App, model: &mut Model) {
    // floating towards mouse position
    model.center_x += (app.mouse().x - model.center_x) * 0.01;
    model.center_y += (app.mouse().y - model.center_y) * 0.01;

    // calculate new points
    for i in 0..model.form_resolution {
        model.x[i] += random_range(-model.step_size, model.step_size);
        model.y[i] += random_range(-model.step_size, model.step_size);
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    if app.elapsed_frames() == 0 || app.keys().just_pressed(KeyCode::Delete) {
        draw.background().color(WHITE);
    }

    let mut builder = nannou::geom::path::Builder::new().with_svg();

    // TODO implement the Catmull–Rom spline algo in lyon, see curveVertex() in Processing
    // first control point
    builder.move_to(lyon::math::point(
        model.x[model.form_resolution - 1] + model.center_x,
        model.y[model.form_resolution - 1] + model.center_y,
    ));
    // only these points are drawn
    for i in 0..model.form_resolution {
        builder.quadratic_bezier_to(
            lyon::math::point(model.x[i] + model.center_x, model.y[i] + model.center_y),
            lyon::math::point(model.x[i] + model.center_x, model.y[i] + model.center_y),
        );
    }
    builder.quadratic_bezier_to(
        lyon::math::point(model.x[0] + model.center_x, model.y[0] + model.center_y),
        lyon::math::point(model.x[0] + model.center_x, model.y[0] + model.center_y),
    );
    // end control point
    builder.move_to(lyon::math::point(
        model.x[1] + model.center_x,
        model.y[1] + model.center_y,
    ));
    builder.close();
    let path = builder.build();

    if model.filled {
        let gray = random_f32();
        draw.path()
            .fill()
            .rgba(gray, gray, gray, 0.4)
            .events(path.iter());
    } else {
        draw.path()
            .stroke()
            .rgba(0.0, 0.0, 0.0, 0.4)
            .events(path.iter());
    }

    // Write to the window frame.

}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::KeyS => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        KeyCode::Digit1 => {
            model.filled = false;
        }
        KeyCode::Digit2 => {
            model.filled = true;
        }
        KeyCode::F => {
            model.freeze = !model.freeze;
            if model.freeze {
                app.set_loop_mode(LoopMode::loop_once());
            } else {
                app.set_loop_mode(LoopMode::RefreshSync);
            }
        }
        _ => (),
    }
}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    // init shape on mouse position
    model.center_x = app.mouse().x;
    model.center_y = app.mouse().y;
    let angle = (360.0 / model.form_resolution as f32).to_radians();
    let _radius = model.init_radius * random_range(0.5, 1.0);
    for i in 0..model.form_resolution {
        model.x[i] = (angle * i as f32).cos() * model.init_radius;
        model.y[i] = (angle * i as f32).sin() * model.init_radius;
    }
}
