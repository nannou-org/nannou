// M_6_1_01
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
 * distribute nodes on the display by letting them repel each other
 *
 * KEYS
 * r             : reset positions
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
    radius: f32,   // Radius of impact
    ramp: f32,     // Influences the shape of the function
    strength: f32, // Strength: positive value attracts, negative value repels
    damping: f32,
    velocity: Vector2,
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
            radius: 200.0,
            ramp: 1.0,
            strength: -1.0,
            damping: 0.5,
            velocity: vec2(0.0, 0.0),
            max_velocity: 10.0,
        }
    }

    fn update(&mut self) {
        self.velocity = self.velocity.limit_magnitude(self.max_velocity);

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
    nodes: Vec<Node>,
    node_count: usize,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1280, 720)
        .view(view)
        .key_released(key_released)
        .build()
        .unwrap();

    let node_count = 200;
    let nodes = create_nodes(node_count, app.window_rect());

    Model { nodes, node_count }
}

fn create_nodes(node_count: usize, win: Rect) -> Vec<Node> {
    (0..node_count)
        .map(|_| {
            Node::new(
                random_range(-1.0, 1.0),
                random_range(-1.0, 1.0),
                win.left() + 5.0,
                win.right() - 5.0,
                win.top() - 5.0,
                win.bottom() + 5.0,
            )
        })
        .collect()
}

fn attract_nodes(nodes: &mut Vec<Node>, target: usize) {
    for other in 0..nodes.len() {
        // Continue from the top when node is itself
        if other == target {
            continue;
        }
        let df = attract(&nodes[target], &nodes[other]);
        nodes[other].velocity += df;
    }
}

fn attract(current_node: &Node, other_node: &Node) -> Vector2 {
    let current_node_vector = vec2(current_node.x, current_node.y);
    let other_node_vector = vec2(other_node.x, other_node.y);
    let d = current_node_vector.distance(other_node_vector);

    if d > 0.0 && d < current_node.radius {
        let s = (d / current_node.radius).powf(1.0 / current_node.ramp);
        let f = s * 9.0 * current_node.strength * (1.0 / (s + 1.0) + ((s - 3.0) / 4.0)) / d;
        let mut df = current_node_vector - other_node_vector;
        df *= f;
        df
    } else {
        vec2(0.0, 0.0)
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    for i in 0..model.nodes.len() {
        // Let all nodes repel each other
        attract_nodes(&mut model.nodes, i);
        // Apply velocity vector and update position
        model.nodes[i].update();
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    if frame.nth() == 0 || app.keys.down.contains(&Key::R) {
        draw.background().color(WHITE);
    } else {
        draw.rect()
            .wh(app.window_rect().wh())
            .rgba(1.0, 1.0, 1.0, 0.07);
    }

    model.nodes.iter().for_each(|node| {
        draw.ellipse().x_y(node.x, node.y).radius(5.0).color(BLACK);
    });

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn key_released(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => model.nodes = create_nodes(model.node_count, app.window_rect()),
        Key::S => {
            app.main_window()
                .capture_frame(app.exe_name().unwrap() + ".png");
        }
        _other_key => {}
    }
}
