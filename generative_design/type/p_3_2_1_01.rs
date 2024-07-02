// P_3_2_1_01
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

// CREDITS
// FreeSans.otf (GNU FreeFont), see the readme files in data folder.

/**
 * typo outline displayed as dots and lines
 *
 * KEYS
 * a-z                  : text input (keyboard)
 * backspace            : delete last typed letter
 * ctrl                 : save png
 */
use nannou::lyon;
use nannou::lyon::algorithms::path::math::Point;
use nannou::lyon::algorithms::path::PathSlice;
use nannou::lyon::algorithms::walk::{walk_along_path, RepeatedPattern, WalkerEvent};
use nannou::lyon::path::iterator::*;
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    text_typed: String,
    letter: Option<KeyCode>,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1280, 350)
        .view(view)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build();

    Model {
        text_typed: "Nannou is Amazing!".to_string(),
        letter: None,
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let win_rect = app.main_window().rect().pad_left(20.0);
    let text = text(&model.text_typed)
        .font_size(128)
        .left_justify()
        .build(win_rect);

    let mut builder = lyon::path::Path::builder();
    for e in text.path_events() {
        builder.path_event(e);
    }
    let path = builder.build();

    let mut path_points: Vec<lyon::path::math::Point> = Vec::new();
    dots_along_path(
        path.as_slice(),
        &mut path_points,
        12.5,
        app.elapsed_frames() as f32,
    );

    path_points.iter().enumerate().for_each(|(i, p)| {
        //Lines
        let l = 5.0;
        draw.line()
            .start(pt2(p.x + l, p.y - l))
            .end(pt2(p.x - l, p.y + l))
            .srgb(0.7, 0.61, 0.0);
        // Dots
        let q = map_range(i, 0, path_points.len(), 0.0, 1.0);
        if i % 2 == 0 {
            draw.ellipse()
                .x_y(p.x, p.y)
                .radius(map_range(
                    (i as f32 * 0.05 + app.elapsed_seconds() * 4.3).sin(),
                    -1.0,
                    1.0,
                    3.0,
                    9.0,
                ))
                .hsv(q, 1.0, 0.5);
        }
    });
}

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    model.letter = key.into();
}
fn key_released(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::ControlLeft || key == KeyCode::ControlRight {
        app.main_window()
            .save_screenshot(app.exe_name().unwrap() + ".png");
    }
}

fn dots_along_path(path: PathSlice, dots: &mut Vec<Point>, interval: f32, offset: f32) {
    use std::ops::Rem;
    let dot_spacing = map_range(interval, 0.0, 1.0, 0.025, 1.0);
    let mut pattern = RepeatedPattern {
        callback: &mut |evt: WalkerEvent| {
            let position = evt.position;
            dots.push(position);
            true // Return true to continue walking the path.
        },
        // Invoke the callback above at a regular interval of 3 units.
        intervals: &[dot_spacing], // 0.05],// 0.05],
        index: 0,
    };

    let tolerance = 0.01; // The path flattening tolerance.
    let start_offset = offset.rem(12.0 + dot_spacing); // Start walking at the beginning of the path.
    walk_along_path(
        path.iter().flattened(tolerance),
        start_offset,
        tolerance,
        &mut pattern,
    );
}
