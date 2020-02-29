// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A type for a draggable attractive body in our world
struct Attractor {
    mass: f32,            // Mass, tied to size
    g: f32,               // Gravitational Constant
    position: Point2,     // Position
    dragging: bool,       // Is the object being dragged?
    rollover: bool,       // Is the mouse over the ellipse?
    drag_offset: Vector2, // Holds the offset for when the object is clicked on
}

impl Attractor {
    fn new() -> Self {
        Attractor {
            mass: 20.0,
            g: 1.0,
            position: pt2(0.0, 0.0),
            dragging: false,
            rollover: false,
            drag_offset: pt2(0.0, 0.0),
        }
    }

    fn attract(&self, m: &Mover) -> Vector2<f32> {
        let mut force = self.position - m.position; // Calculate direction of force
        let mut distance = force.magnitude(); // Distance between objects
        distance = distance.max(5.0).min(25.0); // Limiting the distance to eliminate "extreme" results for very cose or very far object
        force = force.normalize(); // Normalize vector (distance doesn't matter, we just want this vector for direction)
        let strength = (self.g * self.mass * m.mass) / (distance * distance); // Calculate gravitational force magnitude
        force * strength // Get force vector --> magnitude * direction
    }

    // Method to display
    fn display(&self, draw: &app::Draw) {
        let c = if self.dragging {
            rgba(0.2, 0.2, 0.2, 1.0)
        } else if self.rollover {
            rgba(0.4, 0.4, 0.4, 1.0)
        } else {
            rgba(0.7, 0.7, 0.7, 0.78)
        };
        draw.ellipse()
            .xy(self.position)
            .radius(self.mass)
            .color(c)
            .stroke(BLACK)
            .stroke_weight(4.0);
    }

    fn clicked(&mut self, mx: f32, my: f32) {
        let d = pt2(mx, my).distance(self.position);
        if d < self.mass {
            self.dragging = true;
            self.drag_offset.x = self.position.x - mx;
            self.drag_offset.y = self.position.y - my;
        }
    }

    fn hover(&mut self, mx: f32, my: f32) {
        let d = pt2(mx, my).distance(self.position);
        if d < self.mass {
            self.rollover = true;
        } else {
            self.rollover = false;
        }
    }

    fn stop_dragging(&mut self) {
        self.dragging = false;
    }

    fn drag(&mut self, mx: f32, my: f32) {
        if self.dragging {
            self.position.x = mx + self.drag_offset.x;
            self.position.y = my - self.drag_offset.y;
        }
    }
}

struct Mover {
    position: Point2,
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    mass: f32,
}

impl Mover {
    fn new() -> Self {
        Mover {
            position: pt2(80.0, 130.0),
            velocity: pt2(1.0, 0.0),
            acceleration: pt2(0.0, 0.0),
            mass: 1.0,
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
        let heading = (self.velocity.angle() + PI / 2.0) * -1.0;
        draw.ellipse()
            .xy(self.position)
            .w_h(16.0, 16.0)
            .color(GREY)
            .stroke(BLACK)
            .stroke_weight(2.0)
            .rotate(heading);

        draw.rect()
            .x_y(self.position.x + 20.0, self.position.y)
            .w_h(10.0, 10.0)
            .color(GREY)
            .stroke(BLACK)
            .stroke_weight(2.0)
            .rotate(heading);
    }
}

struct Model {
    mover: Mover,
    attractor: Attractor,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(640, 360)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build()
        .unwrap();

    Model {
        mover: Mover::new(),
        attractor: Attractor::new(),
    }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    let force = m.attractor.attract(&m.mover);
    m.mover.apply_force(force);
    m.mover.update();

    m.attractor.drag(app.mouse.x, app.mouse.y);
    m.attractor.hover(app.mouse.x, app.mouse.y);
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.attractor.display(&draw);
    m.mover.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(app: &App, m: &mut Model, _button: MouseButton) {
    m.attractor.clicked(app.mouse.x, app.mouse.y);
}

fn mouse_released(_app: &App, m: &mut Model, _button: MouseButton) {
    m.attractor.stop_dragging();
}
