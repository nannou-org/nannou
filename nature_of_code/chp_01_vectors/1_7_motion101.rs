// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-7: Motion 101
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    mover: Mover,
}

struct Mover {
    position: Point2,
    velocity: Vec2,
}

impl Mover {
    fn new(rect: geom::Rect) -> Self {
        let position = pt2(
            random_range(rect.left(), rect.right()),
            random_range(rect.top(), rect.bottom()),
        );
        let velocity = vec2(random_range(-2.0, 2.0), random_range(-2.0, 2.0));
        Mover { position, velocity }
    }

    fn update(&mut self) {
        // Add the current speed to the position.
        self.position += self.velocity;
    }

    fn check_edges(&mut self, rect: Rect) {
        if self.position.x > rect.right() {
            self.position.x = rect.left();
        } else if self.position.x < rect.left() {
            self.position.x = rect.right();
        }
        if self.position.y > rect.top() {
            self.position.y = rect.bottom();
        } else if self.position.y < rect.bottom() {
            self.position.y = rect.top();
        }
    }

    fn display(&self, draw: &Draw) {
        // Display circle at x position
        draw.ellipse()
            .xy(self.position)
            .w_h(48.0, 48.0)
            .gray(0.5)
            .stroke(Color::gray(0.0));
    }
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();
    let mover = Mover::new(app.window_rect());
    Model { mover }
}

fn update(app: &App, m: &mut Model) {
    m.mover.update();
    m.mover.check_edges(app.window_rect());
}

fn view(app: &App, m: &Model) {
    // Begin drawing
    let draw = app.draw();
    draw.rect()
        .wh(app.window_rect().wh())
        .rgba(1.0, 1.0, 1.0, 0.03);

    m.mover.display(&draw);
}
