// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-9: Motion 101 Acceleration
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    mover: Mover,
}

struct Mover {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
    top_speed: f32,
}

impl Mover {
    fn new(_rect: Rect) -> Self {
        let position = pt2(0.0, 0.0);
        let velocity = vec2(0.0, 0.0);
        let acceleration = vec2(0.0, 0.0);
        let top_speed = 6.0;
        Mover {
            position,
            velocity,
            acceleration,
            top_speed,
        }
    }

    fn update(&mut self) {
        self.acceleration = vec2(1.0 - random_f32() * 2.0, 1.0 - random_f32() * 2.0);
        self.acceleration *= random_f32() * 2.0;
        self.velocity += self.acceleration;
        self.velocity = vec2(
            self.velocity.x.min(self.top_speed),
            self.velocity.y.min(self.top_speed),
        );
        self.position += self.velocity;
    }

    fn _check_edges(&mut self, rect: Rect) {
        if self.position.x > rect.right() {
            self.position.x = rect.left();
        } else if self.position.x < rect.left() {
            self.position.x = rect.right();
        }
        if self.position.y > rect.top() {
            self.position.y = rect.bottom();
        } else if self.position.y < rect.bottom() {
            self.position.y = rect.top();;
        }
    }

    fn display(&self, draw: &app::Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    let _window = app.new_window().with_dimensions(640, 360).build().unwrap();
    let mover = Mover::new(app.window_rect());
    Model { mover }
}

fn event(_app: &App, mut m: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        m.mover.update();
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.mover.display(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
