// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-7: Motion 101
extern crate nannou;

use nannou::prelude::*;
use nannou::app::Draw;
use nannou::geom::rect::Rect;
use nannou::rand::random;
use nannou::math::map_range;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
    mover: Mover,
}

struct Mover {
    position: Point2<f32>,
    velocity: Vector2<f32>,
}

impl Mover {
    fn new(rect: Rect<f32>) -> Self {
        let position = Point2::new(map_range(random(),0.0,1.0,rect.left(),rect.right()),
        map_range(random(),0.0,1.0,rect.top(),rect.bottom()));
        let velocity = Vector2::new(map_range(random(),0.0,1.0,-2.0,2.0), map_range(random(),0.0,1.0,-2.0,2.0));
        Mover { position, velocity }
    }

    fn update(&mut self) {
        // Add the current speed to the position.
        self.position += self.velocity;
    }

    fn check_edges(&mut self, rect:  Rect<f32>){
        if self.position.x > rect.right() {
            self.position.x = rect.left();
        }
        else if self.position.x < rect.left() {
            self.position.x = rect.right();
        }
        if self.position.y > rect.top() {
            self.position.y = rect.bottom();
        }
        else if self.position.y < rect.bottom() {
            self.position.y = rect.top();;
        }
    }

    fn display(&self, draw: &Draw) {
        // Display circle at x position
        draw.ellipse()
            .x_y(self.position.x, self.position.y)
            .w_h(16.0, 16.0)
            .rgb(0.5, 0.5, 0.5);
    }
}

fn model(app: &App) -> Model {
    let window = app.new_window().with_dimensions(640,360).build().unwrap();
    let mover = Mover::new(app.window.rect());
    Model { window, mover }
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            match event {
                // KEY EVENTS
                KeyPressed(_key) => {}

                // MOUSE EVENTS
                MouseReleased(_button) => {}

                _other => (),
            }
        }
        // update gets called just before view every frame
        Event::Update(_dt) => {
            m.mover.update();
            m.mover.check_edges(app.window.rect());
        }
        _ => (),
    }
    m
}

fn view(app: &App, m: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("noc_1_7_motion101");

    // Begin drawing
    let draw = app.draw();
    draw.rect().wh(app.window.size()).rgba(1.0, 1.0, 1.0, 0.03);

    m.mover.display(&draw);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
