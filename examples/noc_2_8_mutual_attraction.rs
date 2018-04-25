// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 2-8: Mutual Attraction 
extern crate nannou;

use nannou::prelude::*;
use nannou::app::Draw;
use nannou::geom::rect::Rect;
use nannou::rand::random;
use nannou::math::prelude::*;
use nannou::math::map_range;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
    movers: Vec<Mover>,
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
    
    fn attract(&self, m: &Mover) -> Vector2<f32> {
        let mut force = self.position - m.position;                      // Calculate direction of force
        let mut distance = force.magnitude();                            // Distance between objects
        distance = distance.max(5.0).min(25.0);                          // Limiting the distance to eliminate "extreme" results for very cose or very far object
        force = force.normalize();                                       // Normalize vector (distance doesn't matter, we just want this vector for direction)
        let g = 0.4;
        let strength = (g * self.mass * m.mass) / (distance * distance); // Calculate gravitational force magnitude
        force * strength                                                 // Get force vector --> magnitude * direction
    }

    fn display(&self, draw: &Draw) {
        draw.ellipse()
            .x_y(self.position.x, self.position.y)
            .w_h(self.mass * 24.0, self.mass * 24.0)
            .rgba(0.0, 0.0, 0.0, 0.5);
    }
}

fn model(app: &App) -> Model {
    let rect = Rect::from_wh(Vector2::new(640.0, 360.0));
    let window = app.new_window()
        .with_dimensions(rect.w() as u32, rect.h() as u32)
        .build()
        .unwrap();

    let movers = (0..1000)
        .map(|_| {
            Mover::new(
                map_range(random(), 0.0, 1.0, 0.1, 2.0),
                map_range(random(), 0.0, 1.0, rect.left(), rect.right()),
                map_range(random(), 0.0, 1.0, rect.top(), rect.bottom()),
            )
        })
        .collect();
    
    Model {
        window,
        movers,
    }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            match event {
                // MOUSE EVENTS
                MousePressed(_button) => {
                }
                MouseReleased(_buttom) => {
                }
                _other => (),
            }
        }
        // update gets called just before view every frame
        Event::Update(_dt) => {
            for i in 0..m.movers.len() {
                for j in 0..m.movers.len() {
                    if i != j {
                        let force = m.movers[j].attract(&m.movers[i]);
                        m.movers[i].apply_force(force);
                    }
                }
                m.movers[i].update();
            }
        }
        _ => (),
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    app.main_window()
        .set_title("noc_2_8_mutual_attraction");

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
