// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// A static drawing of a Neural Network
//
// Example 10-3: Network Viz
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

#[derive(Clone)]
struct Neuron {
    position: Vector2,            // Neuron has a position
    connections: Vec<Connection>, //Neuron has a list of connections
}

impl Neuron {
    fn new(x: f32, y: f32) -> Self {
        Neuron {
            position: vec2(x, y),
            connections: Vec::new(),
        }
    }

    // Add a connection
    fn add_connection(&mut self, c: Connection) {
        self.connections.push(c);
    }

    // Draw a neuron as a circle
    fn display(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.position)
            .radius(8.0)
            .color(BLACK)
            .stroke(BLACK);

        for c in &self.connections {
            c.display(&draw);
        }
    }
}

#[derive(Clone)]
struct Connection {
    // Connection is from Neuron A to B
    a: Neuron,
    b: Neuron,
    weight: f32, // Connection has a weight
}

impl Connection {
    fn new(from: Neuron, to: Neuron, w: f32) -> Self {
        Connection {
            weight: w,
            a: from,
            b: to,
        }
    }

    // Draw as a line
    fn display(&self, draw: &Draw) {
        draw.line()
            .start(self.a.position)
            .end(self.b.position)
            .color(BLACK)
            .stroke_weight(self.weight * 4.0);
    }
}

struct Network {
    neurons: Vec<Neuron>, // The network has a list of neurons
}

impl Network {
    fn new() -> Self {
        Network {
            neurons: Vec::new(),
        }
    }

    // We can add a Neuron
    fn add_neuron(&mut self, n: Neuron) {
        self.neurons.push(n);
    }

    // We can connect two Neurons
    fn connect(&mut self, a: &mut Neuron, b: &Neuron) {
        let c = Connection::new(a.clone(), b.clone(), random_f32());
        a.add_connection(c);
    }

    // We can draw the network
    fn display(&self, draw: &Draw) {
        for n in &self.neurons {
            n.display(&draw);
        }
    }
}

struct Model {
    network: Network,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();

    // Create an empty network
    let mut network = Network::new();

    // Create a bunch of neurons
    let mut a = Neuron::new(-200.0, 0.0);
    let mut b = Neuron::new(0.0, 75.0);
    let mut c = Neuron::new(0.0, -75.0);
    let d = Neuron::new(200.0, 0.0);

    // Connect them
    network.connect(&mut a, &b);
    network.connect(&mut a, &c);
    network.connect(&mut b, &d);
    network.connect(&mut c, &d);

    // Add them to the network
    network.add_neuron(a);
    network.add_neuron(b);
    network.add_neuron(c);
    network.add_neuron(d);

    Model { network }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    model.network.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
