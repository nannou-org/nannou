// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// A static drawing of a Neural Network
//
// Example 10-3: Network Viz
use nannou::geom::range::Range;
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone)]
struct Neuron {
    position: Vector2,                // Neuron has a position
    pub connections: Vec<Connection>, // Neuron has a list of connections
    sum: f32,                         // We now track the inputs and sum them
    r: f32,                           // The Neuron's size can be animated
}

impl Neuron {
    fn new(x: f32, y: f32) -> Self {
        Neuron {
            position: vec2(x, y),
            connections: Vec::new(),
            sum: 0.0,
            r: 32.0,
        }
    }

    // Add a connection
    fn add_connection(&mut self, c: Connection) {
        self.connections.push(c);
    }

    // Receive an input
    fn feedforward(&mut self, input: f32) {
        // Accumulate it
        self.sum += input;
        // Activate it?
        if self.sum > 1.0 {
            self.fire();
            self.sum = 0.0; // Reset the sum to 0 if it fires
        }
    }

    // The Neuron fires
    fn fire(&mut self) {
        self.r = 64.0; // It suddenly is bigger

        // We send the output through all connections
        for c in &mut self.connections {
            c.feedforward(self.sum);
        }
    }

    fn update(&mut self) {
        // Size shrinks down back to original dimensions
        self.r = Range::new(self.r, 32.0).lerp(0.1);
    }

    // Draw a neuron as a circle
    fn display(&self, draw: &app::Draw) {
        let b = 1.0 - self.sum; // Brightness is mapped to sum
        draw.ellipse()
            .xy(self.position)
            .radius(self.r / 2.0)
            .rgb(b, b, b)
            .stroke(BLACK);
    }
}

#[derive(Clone)]
struct Connection {
    // Connection is from Neuron A to B
    a: Neuron,
    b: Neuron,
    // Connection has a weight
    weight: f32,
    // Variables to track the animation
    sending: bool,
    sender: Vector2,
    // Need to store the output for when its time to pass along
    output: f32,
}

impl Connection {
    fn new(from: Neuron, to: Neuron, w: f32) -> Self {
        Connection {
            a: from,
            b: to,
            weight: w,
            sending: false,
            sender: vec2(0.0, 0.0),
            output: 0.0,
        }
    }

    fn feedforward(&mut self, val: f32) {
        self.output = val * self.weight; // Compute output
        self.sender = self.a.position; // Start animation at Neuron A
        self.sending = true; // Turn on sending
    }

    // Update traveling sender
    fn update(&mut self) {
        if self.sending {
            //Use simple interpolation
            self.sender.x = Range::new(self.sender.x, self.b.position.x).lerp(0.1);
            self.sender.y = Range::new(self.sender.y, self.b.position.y).lerp(0.1);
            let d = self.sender.distance(self.b.position);
            // If we've reached the end
            if d < 1.0 {
                // Pass along the output
                self.b.feedforward(self.output);
                self.sending = false;
            }
        }
    }

    // Draw line and traveling circle
    fn display(&self, draw: &app::Draw) {
        draw.line()
            .start(self.a.position)
            .end(self.b.position)
            .color(BLACK)
            .stroke_weight(1.0 + self.weight * 4.0);

        if self.sending {
            draw.ellipse().xy(self.sender).radius(8.0).color(BLACK);
        }
    }
}

struct Network {
    // The network has a list of neurons
    neurons: Vec<Neuron>,
    // The Network now keeps a duplicate list of all Connection objects.
    // This makes it easier to draw everything in this class
    connections: Vec<Connection>,
}

impl Network {
    fn new() -> Self {
        Network {
            neurons: Vec::new(),
            connections: Vec::new(),
        }
    }

    // We can add a Neuron
    fn add_neuron(&mut self, n: Neuron) {
        self.neurons.push(n);
    }

    // We can connect two Neurons
    fn connect(&mut self, a: &mut Neuron, b: &Neuron, weight: f32) {
        let c = Connection::new(a.clone(), b.clone(), weight);
        // Also add the Connection here
        self.connections.push(c.clone());
        a.add_connection(c);
    }

    // Sending an input to the first Neuron
    // We should do something better to track multiple inputs
    fn feedforward(&mut self, input: f32) {
        self.neurons[0].feedforward(input);
    }

    // Update the animation
    fn update(&mut self) {
        for n in &mut self.neurons {
            n.update();
        }

        for c in &mut self.connections {
            c.update();
        }
    }

    // We can draw the network
    fn display(&self, draw: &app::Draw) {
        for n in &self.neurons {
            n.display(&draw);
        }
        for c in &self.connections {
            c.display(&draw);
        }
    }
}

struct Model {
    network: Network,
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(640, 360)
        .view(view)
        .build()
        .unwrap();

    // Create an empty network
    let mut network = Network::new();

    // Create a bunch of neurons
    let mut a = Neuron::new(-275.0, 0.0);
    let mut b = Neuron::new(-150.0, 0.0);
    let mut c = Neuron::new(0.0, 75.0);
    let mut d = Neuron::new(0.0, -75.0);
    let mut e = Neuron::new(150.0, 0.0);
    let f = Neuron::new(275.0, 0.0);

    // Connect them
    network.connect(&mut a, &b, 1.0);
    network.connect(&mut b, &c, random_f32());
    network.connect(&mut b, &d, random_f32());
    network.connect(&mut c, &e, random_f32());
    network.connect(&mut d, &e, random_f32());
    network.connect(&mut e, &f, 1.0);

    // Add them to the network
    network.add_neuron(a);
    network.add_neuron(b);
    network.add_neuron(c);
    network.add_neuron(d);
    network.add_neuron(e);
    network.add_neuron(f);

    Model { network }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.network.update();

    // Every 30 frames feed in an input
    if app.elapsed_frames() % 30 == 0 {
        model.network.feedforward(random_f32());
    }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    model.network.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
