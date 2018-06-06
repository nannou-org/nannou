// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 2-0 Many Mutual Boundaries
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Mover {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    mass: f32,
}

impl Mover {
    fn new(m: f32, x: f32, y: f32) -> Self {
        let mass = m;
        let position = Point2::new(x, y);
        let velocity = Vector2::new(0.0, 0.0);
        let acceleration = Vector2::new(0.0, 0.0);
        Mover {
            position,
            velocity,
            acceleration,
            mass,
        }
    }

    fn apply_force(&mut self, force: Vector2<f32>) {
        let f = force / self.mass;
        self.acceleration += f;
    }

    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position += self.velocity;
        self.acceleration *= 0.0;
    }

    fn display(&self, draw: &app::Draw) {
        draw.ellipse()
            .x_y(self.position.x, self.position.y)
            .w_h(self.mass * 16.0, self.mass * 16.0)
            .rgba(0.6, 0.6, 0.6, 0.7);
    }

    fn attract(&self, m: &Mover) -> Vector2<f32> {
        let mut force = self.position - m.position; // Calculate direction of force
        let mut distance = force.magnitude(); // Distance between objects
        distance = distance.max(5.0).min(25.0); // Limiting the distance to eliminate "extreme" results for very cose or very far object
        force = force.normalize(); // Normalize vector (distance doesn't matter, we just want this vector for direction)
        let g = 0.4;
        let strength = (g * self.mass * m.mass) / (distance * distance); // Calculate gravitational force magnitude
        force * strength // Get force vector --> magnitude * direction
    }

    fn boundaries(&mut self, rect: Rect<f32>) {
        let d = 50.0;
        let mut force = Vector2::new(0.0, 0.0);

        if self.position.x < rect.pad(d).left() {
            force.x = 1.0;
        } else if self.position.x > rect.pad(d).right() {
            force.x = -1.0;
        }
        if self.position.y > rect.pad(d).top() {
            force.y = -1.0;
        } else if self.position.y < rect.pad(d).bottom() {
            force.y = 1.0;
        }
        if force.magnitude() > 0.0 {
            force = force.normalize();
            force *= 0.1;
            self.apply_force(force);
        }
    }
}

struct Model {
    movers: Vec<Mover>,
}

fn model(app: &App) -> Model {
    let rect = Rect::from_wh(Vector2::new(640.0, 360.0));
    let _window = app.new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .build()
        .unwrap();

    let movers = (0..20)
        .map(|_| {
            Mover::new(
                map_range(random(), 0.0, 1.0, 1.0, 2.0),
                map_range(random(), 0.0, 1.0, rect.left(), rect.right()),
                map_range(random(), 0.0, 1.0, rect.top(), rect.bottom()),
            )
        })
        .collect();

    Model { movers }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        for i in 0..m.movers.len() {
            for j in 0..m.movers.len() {
                if i != j {
                    let force = m.movers[j].attract(&m.movers[i]);
                    m.movers[i].apply_force(force);
                }
            }
            m.movers[i].boundaries(app.window_rect());
            m.movers[i].update();
        }
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    // Draw movers
    for mover in &m.movers {
        mover.display(&draw);
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
