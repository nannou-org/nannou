// M_1_5_04
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
 * noise values (noise 3d) are used to animate a bunch of agents.
 *
 * KEYS
 * 1                   : draw style line
 * 2                   : draw style ellipse
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
    randomizer: f32,
    step_size: f32,
    z_noise: f32,
    angle: f32,
    color: Hsla,
    noise_scale: f64,
    noise_strength: f64,
    agent_width: f32,
    agent_width_min: f32,
    agent_width_max: f32,
    noise_z_velocity: f64,
    win_rect: Rect,
}

impl Agent {
    fn new(
        win_rect: Rect,
        noise_sticking_range: f32,
        agent_alpha: f32,
        noise_scale: f64,
        noise_strength: f64,
        agent_width_min: f32,
        agent_width_max: f32,
        noise_z_velocity: f64,
    ) -> Self {
        let vector = vec2(
            random_range(win_rect.left(), win_rect.right()),
            random_range(win_rect.top(), win_rect.bottom()),
        );
        let randomizer = random_f32();
        let color = if randomizer < 0.5 {
            Color::hsla(random_range(0.47, 0.52), 0.7, random_f32(), agent_alpha)
        } else {
            Color::hsla(random_range(0.11, 0.16), 0.7, random_f32(), agent_alpha)
        };
        Agent {
            vector,
            vector_old: vector,
            randomizer,
            step_size: 1.0 + randomizer * 4.0,
            z_noise: random_f32() * noise_sticking_range,
            angle: 0.0,
            color,
            noise_scale,
            noise_strength,
            agent_width: agent_width_min,
            agent_width_min,
            agent_width_max,
            noise_z_velocity,
            win_rect,
        }
    }

    fn update(&mut self, noise: Perlin, draw_mode: u8) {
        self.vector_old = self.vector;
        self.z_noise += self.noise_z_velocity as f32;

        let n = noise.get([
            self.vector.x as f64 / self.noise_scale,
            self.vector.y as f64 / self.noise_scale,
            self.z_noise as f64,
        ]) * self.noise_strength;
        self.angle = n as f32;

        self.vector.x += self.angle.cos() * self.step_size;
        self.vector.y += self.angle.sin() * self.step_size;

        if self.vector.x < self.win_rect.left() - 10.0 {
            self.vector.x = self.win_rect.right() + 10.0;
            self.vector_old.x = self.vector.x;
        }
        if self.vector.x > self.win_rect.right() + 10.0 {
            self.vector.x = self.win_rect.left() - 10.0;
            self.vector_old.x = self.vector.x;
        }
        if self.vector.y < self.win_rect.bottom() - 10.0 {
            self.vector.y = self.win_rect.top() + 10.0;
            self.vector_old.y = self.vector.y;
        }
        if self.vector.y > self.win_rect.top() + 10.0 {
            self.vector.y = self.win_rect.bottom() - 10.0;
            self.vector_old.y = self.vector.y;
        }

        self.agent_width =
            nannou::geom::range::Range::new(self.agent_width_min, self.agent_width_max)
                .lerp(self.randomizer);

        if draw_mode == 2 {
            self.agent_width *= 2.0;
        }
    }

    fn display(&self, draw: &Draw, draw_mode: u8, stroke_weight: f32) {
        if draw_mode == 1 {
            draw.line()
                .start(self.vector_old)
                .end(self.vector)
                .color(self.color)
                .stroke_weight(stroke_weight * self.step_size);

            let draw = draw.x_y(self.vector_old.x, self.vector_old.y).rotate(
                (self.vector.y - self.vector_old.y).atan2(self.vector.x - self.vector_old.x),
            );

            draw.line()
                .start(pt2(0.0, -self.agent_width))
                .end(pt2(0.0, self.agent_width))
                .color(self.color)
                .stroke_weight(stroke_weight * self.step_size);
        } else if draw_mode == 2 {
            draw.ellipse()
                .x_y(self.vector_old.x, self.vector_old.y)
                .radius(self.agent_width / 2.0)
                .stroke_weight(stroke_weight)
                .stroke(self.color)
                .no_fill();
        }
    }
}

struct Model {
    agents: Vec<Agent>,
    overlay_alpha: f32,
    stroke_width: f32,
    draw_mode: u8,
    noise_seed: u32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    let agent_count = 2000;
    let noise_scale = 100.0;
    let noise_strength = 10.0;
    let noise_sticking_range = 0.4;
    let noise_z_velocity = 0.01;
    let agent_alpha = 0.9;
    let agent_width_min = 1.5;
    let agent_width_max = 15.0;
    let agents = (0..agent_count)
        .map(|_| {
            Agent::new(
                app.window_rect(),
                noise_sticking_range,
                agent_alpha,
                noise_scale,
                noise_strength,
                agent_width_min,
                agent_width_max,
                noise_z_velocity,
            )
        })
        .collect();

    Model {
        agents,
        overlay_alpha: 0.08,
        stroke_width: 2.0,
        draw_mode: 1,
        noise_seed: 12,
    }
}

fn update(_app: &App, model: &mut Model) {
    let noise = Perlin::new().set_seed(model.noise_seed);

    for agent in &mut model.agents {
        agent.update(noise, model.draw_mode);
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
            .hsla(0.0, 0.0, 1.0, model.overlay_alpha);
    }

    model.agents.iter().for_each(|agent| {
        agent.display(&draw, model.draw_mode, model.stroke_width);
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
