// M_6_1_02
// Spring.js
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
 * two nodes and a spring
 *
 * MOUSE
 * click, drag   : position of one of the nodes
 *
 * KEYS
 * s             : save png
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Node {
    pub x: f32,
    pub y: f32,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    pub damping: f32,
    pub velocity: Vec2,
    max_velocity: f32,
}

impl Node {
    fn new(x: f32, y: f32, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Node {
            x,
            y,
            min_x,
            max_x,
            min_y,
            max_y,
            damping: 0.1,
            velocity: vec2(0.0, 0.0),
            max_velocity: 10.0,
        }
    }

    fn update(&mut self) {
        self.velocity = self.velocity.clamp_length_max(self.max_velocity);

        self.x += self.velocity.x;
        self.y += self.velocity.y;

        if self.x < self.min_x {
            self.x = self.min_x - (self.x - self.min_x);
            self.velocity.x = -self.velocity.x;
        }
        if self.x > self.max_x {
            self.x = self.max_x - (self.x - self.max_x);
            self.velocity.x = -self.velocity.x;
        }

        if self.y < self.min_y {
            self.y = self.min_y + (self.y - self.min_y);
            self.velocity.y = -self.velocity.y;
        }
        if self.y > self.max_y {
            self.y = self.max_y + (self.y - self.max_y);
            self.velocity.y = -self.velocity.y;
        }

        self.velocity *= 1.0 - self.damping;
    }
}

struct Model {
    node_a: Node,
    node_b: Node,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    let win = app.window_rect();
    let node_a = Node::new(
        random_range(-50.0, 50.0),
        random_range(-50.0, 50.0),
        win.left() + 5.0,
        win.right() - 5.0,
        win.top() - 5.0,
        win.bottom() + 5.0,
    );
    let node_b = Node::new(
        random_range(-50.0, 50.0),
        random_range(-50.0, 50.0),
        win.left() + 5.0,
        win.right() - 5.0,
        win.top() - 5.0,
        win.bottom() + 5.0,
    );

    Model { node_a, node_b }
}

// ------ apply forces on spring and attached nodes ------
fn spring(to_node: &mut Node, from_node: &mut Node) {
    let length = 100.0;
    let stiffness = 0.6;
    let damping = 0.3;

    let mut diff = vec2(to_node.x, to_node.y) - vec2(from_node.x, from_node.y);
    diff = diff.normalize();
    diff *= length;
    let target = vec2(from_node.x, from_node.y) + diff;

    let mut force = target - vec2(to_node.x, to_node.y);
    force *= 0.5;
    force *= stiffness;
    force *= 1.0 - damping;

    to_node.velocity += force;
    force *= -1.0;
    from_node.velocity += force;
}

fn update(app: &App, model: &mut Model) {
    // update spring
    spring(&mut model.node_a, &mut model.node_b);

    // update node positions
    model.node_a.update();
    model.node_b.update();

    if app.mouse.buttons.pressed().next().is_some() {
        model.node_a.x = app.mouse().x;
        model.node_a.y = app.mouse().x;
    }
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    // draw spring
    draw.line()
        .start(pt2(model.node_a.x, model.node_a.y))
        .end(pt2(model.node_b.x, model.node_b.y))
        .stroke_weight(4.0)
        .rgb(0.0, 0.5, 0.64);

    // draw nodes
    draw.ellipse()
        .x_y(model.node_a.x, model.node_a.y)
        .radius(10.0)
        .color(BLACK);
    draw.ellipse()
        .x_y(model.node_b.x, model.node_b.y)
        .radius(10.0)
        .color(BLACK);



}

fn key_released(app: &App, _model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
}
