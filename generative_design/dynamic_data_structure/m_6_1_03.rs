// M_6_1_03
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
 * more nodes and more springs
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
            radius: 100.0,
            ramp: 1.0,
            strength: -5.0,
            damping: 0.5,
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
    nodes: Vec<Node>,
    spring_connections: Vec<(usize, usize)>,
    node_radius: f32,
    node_count: usize,
    selected_node: Option<usize>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .view(view)
        .mouse_released(mouse_released)
        .key_pressed(key_pressed)
        .build();

    let node_radius = 8.0;
    let node_count = 100;
    let nodes = create_nodes(node_count, node_radius, app.window_rect());
    let spring_connections = create_connections(node_count);
    let selected_node = None;

    Model {
        nodes,
        spring_connections,
        node_radius,
        node_count,
        selected_node,
    }
}

fn create_connections(node_count: usize) -> Vec<(usize, usize)> {
    (0..node_count - 1)
        .map(|j| {
            let r = random_range(j + 1, node_count);
            (j, r)
        })
        .collect()
}

fn create_nodes(node_count: usize, node_radius: f32, win: geom::Rect) -> Vec<Node> {
    (0..node_count)
        .map(|_| {
            Node::new(
                random_range(-200.0, 200.0),
                random_range(-200.0, 200.0),
                win.left() + node_radius,
                win.right() - node_radius,
                win.top() - node_radius,
                win.bottom() + node_radius,
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

fn attract(current_node: &Node, other_node: &Node) -> Vec2 {
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

// ------ apply forces on spring and attached nodes ------
fn spring(nodes: &mut Vec<Node>, spring_connection: (usize, usize)) {
    let length = 20.0;
    let stiffness = 1.0;
    let damping = 0.9;

    let (from_node, to_node) = spring_connection;
    let mut diff =
        vec2(nodes[to_node].x, nodes[to_node].y) - vec2(nodes[from_node].x, nodes[from_node].y);
    diff = diff.normalize();
    diff *= length;
    let target = vec2(nodes[from_node].x, nodes[from_node].y) + diff;

    let mut force = target - vec2(nodes[to_node].x, nodes[to_node].y);
    force *= 0.5;
    force *= stiffness;
    force *= 1.0 - damping;

    nodes[to_node].velocity += force;
    force *= -1.0;
    nodes[from_node].velocity += force;
}

fn update(app: &App, model: &mut Model) {
    for i in 0..model.nodes.len() {
        // Let all nodes repel each other
        attract_nodes(&mut model.nodes, i);
    }

    for connection in model.spring_connections.iter() {
        // apply spring forces
        spring(&mut model.nodes, *connection);
    }

    for i in 0..model.nodes.len() {
        // Apply velocity vector and update position
        model.nodes[i].update();
    }

    if app.mouse_buttons().get_just_pressed().count() > 0 {
        // Ignore anything greater than this distance
        let mut max_dist = 20.0;
        for i in 0..model.nodes.len() {
            let d = pt2(app.mouse().x, app.mouse().x).distance(pt2(model.nodes[i].x, model.nodes[i].y));
            if d < max_dist && model.selected_node.is_none() {
                model.selected_node = Some(i);
                max_dist = d;
            }
        }
    }

    if model.selected_node.is_some() {
        model.nodes[model.selected_node.unwrap()].x = app.mouse().x;
        model.nodes[model.selected_node.unwrap()].y = app.mouse().x;
    }
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    model.spring_connections.iter().for_each(|connection| {
        // draw spring
        let (to, from) = *connection;
        draw.line()
            .start(pt2(model.nodes[from].x, model.nodes[from].y))
            .end(pt2(model.nodes[to].x, model.nodes[to].y))
            .stroke_weight(2.0)
            .rgb(0.0, 0.5, 0.64);
    });

    model.nodes.iter().for_each(|node| {
        draw.ellipse()
            .x_y(node.x, node.y)
            .radius(model.node_radius)
            .color(BLACK)
            .stroke(WHITE)
            .stroke_weight(2.0);
    });



}

fn key_pressed(app: &App, model: &mut Model, key: KeyCode) {
    if key == KeyCode::KeyS {
        app.main_window().save_screenshot(app.exe_name().unwrap() + ".png");
    }
    if key == KeyCode::KeyR {
        model.nodes = create_nodes(model.node_count, model.node_radius, app.window_rect());
        model.spring_connections = create_connections(model.node_count);
    }
}

fn mouse_released(_app: &App, model: &mut Model, _button: MouseButton) {
    model.selected_node = None;
}
