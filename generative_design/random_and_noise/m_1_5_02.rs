// M_1_5_02
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
 * noise values (noise 2d) are used to animate a bunch of agents.
 *
 * KEYS
 * 1-2                 : switch noise mode
 * space               : new noise seed
 * backspace           : clear screen
 * s                   : save png
 */
use nannou::noise::{NoiseFn, Perlin, Seedable};
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Agent {
    vector: Vec2,
    vector_old: Vec2,
    step_size: f32,
    angle: f32,
    is_outside: bool,
    win_rect: Rect,
}

impl Agent {
    fn new(win_rect: Rect) -> Self {
        let vector = vec2(
            random_range(win_rect.left(), win_rect.right()),
            random_range(win_rect.top(), win_rect.bottom()),
        );
        Agent {
            vector,
            vector_old: vector,
            step_size: random_range(1.0, 5.0),
            angle: 0.0,
            is_outside: false,
            win_rect,
        }
    }

    fn update(&mut self) {
        self.is_outside = false;
        self.vector_old = self.vector;

        self.vector.x += self.angle.cos() * self.step_size;
        self.vector.y += self.angle.sin() * self.step_size;
        self.is_outside = self.vector.x < self.win_rect.left()
            || self.vector.x > self.win_rect.right()
            || self.vector.y < self.win_rect.bottom()
            || self.vector.y > self.win_rect.top();
        if self.is_outside {
            self.vector = vec2(
                random_range(self.win_rect.left(), self.win_rect.right()),
                random_range(self.win_rect.top(), self.win_rect.bottom()),
            );
            self.vector_old = self.vector;
        }
    }

    fn update1(&mut self, noise: Perlin, noise_scale: f64, noise_strength: f64) {
        let n = noise.get([
            self.vector.x as f64 / noise_scale,
            self.vector.y as f64 / noise_scale,
        ]) * noise_strength;
        self.angle = n as f32;
    }

    fn update2(&mut self, noise: Perlin, noise_scale: f64, noise_strength: f64) {
        let n = noise.get([
            self.vector.x as f64 / noise_scale,
            self.vector.y as f64 / noise_scale,
        ]) * 24.0;
        self.angle = n as f32;
        self.angle = (self.angle - self.angle.floor()) * noise_strength as f32;
    }

    fn display(&self, draw: &Draw, stroke_weight: f32, agent_alpha: f32) {
        draw.line()
            .start(self.vector_old)
            .end(self.vector)
            .rgba(0.0, 0.0, 0.0, agent_alpha)
            .stroke_weight(stroke_weight * self.step_size);
    }
}

struct Model {
    agents: Vec<Agent>,
    noise_scale: f64,
    noise_strength: f64,
    overlay_alpha: f32,
    agent_alpha: f32,
    stroke_width: f32,
    draw_mode: u8,
    noise_seed: u32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(720, 720)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    let agent_count = 4000;
    let agents = (0..agent_count)
        .map(|_| Agent::new(app.window_rect()))
        .collect();

    Model {
        agents,
        noise_scale: 300.0,
        noise_strength: 10.0,
        overlay_alpha: 0.03,
        agent_alpha: 0.35,
        stroke_width: 0.3,
        draw_mode: 1,
        noise_seed: 12,
    }
}

fn update(_app: &App, model: &mut Model) {
    let noise = Perlin::new().set_seed(model.noise_seed);

    for agent in &mut model.agents {
        match model.draw_mode {
            1 => agent.update1(noise, model.noise_scale, model.noise_strength),
            2 => agent.update2(noise, model.noise_scale, model.noise_strength),
            _ => (),
        }
        agent.update();
    }
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();

    if app.elapsed_frames() == 0 || app.keys().just_pressed(KeyCode::Delete) {
        draw.background().color(WHITE);
    } else {
        draw.rect()
            .wh(app.window_rect().wh())
            .rgba(1.0, 1.0, 1.0, model.overlay_alpha);
    }

    model.agents.iter().for_each(|agent| {
        agent.display(&draw, model.stroke_width, model.agent_alpha);
    });



}

fn key_released(app: &App, model: &mut Model, key: KeyCode) {
    match key {
        KeyCode::Digit1 => model.draw_mode = 1,
        KeyCode::Digit2 => model.draw_mode = 2,
        KeyCode::Space => {
            model.noise_seed = (random_f32() * 10000.0).floor() as u32;
        }
        KeyCode::KeyS => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
