// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 2-7: Attraction Many
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    movers: Vec<Mover>,
    attractor: Attractor,
}

struct Mover {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    mass: f32,
}

// A type for a draggable attractive body in our world
struct Attractor {
    mass: f32,         // Maxx, tied to size
    position: Point2,  // position
    dragging: bool,    // Is the object being dragged?
    roll_over: bool,   // Is the mouse over the ellipse?
    drag_offset: Vec2, // holds the offset for when the object is clicked on
}

impl Attractor {
    const G: f32 = 1.0; // Gravitational Constant
    fn new(rect: Rect) -> Self {
        let position = rect.xy();
        let mass = 20.0;
        let drag_offset = vec2(0.0, 0.0);
        let dragging = false;
        let roll_over = false;
        Attractor {
            position,
            mass,
            drag_offset,
            dragging,
            roll_over,
        }
    }

    fn attract(&self, m: &Mover) -> Vec2 {
        let mut force = self.position - m.position; // Calculate direction of force
        let mut d = force.length(); // Distance between objects
        d = d.max(5.0).min(25.0); // Limiting the distance to eliminate "extreme" results for very cose or very far object
        force = force.normalize(); // Normalize vector (distance doesn't matter, we just want this vector for direction)
        let strength = (Attractor::G * self.mass * m.mass) / (d * d); // Calculate gravitational force magnitude
        force * strength // Get force vector --> magnitude * direction
    }

    // Method to display
    fn display(&self, draw: &DrawHolder) {
        let gray = if self.dragging {
            0.2
        } else if self.roll_over {
            0.4
        } else {
            0.75
        };
        draw.ellipse()
            .xy(self.position)
            .w_h(self.mass * 2.0, self.mass * 2.0)
            .rgba(gray, gray, gray, 0.8)
            .stroke(BLACK)
            .stroke_weight(4.0);
    }

    // The methods below are for mouse interaction
    fn clicked(&mut self, mx: f32, my: f32) {
        let d = self.position.distance(pt2(mx, my));
        if d < self.mass {
            self.dragging = true;
            self.drag_offset.x = self.position.x - mx;
            self.drag_offset.y = self.position.y - my;
        }
    }

    fn hover(&mut self, mx: f32, my: f32) {
        let d = self.position.distance(pt2(mx, my));
        if d < self.mass {
            self.roll_over = true;
        } else {
            self.roll_over = false;
        }
    }

    fn stop_dragging(&mut self) {
        self.dragging = false;
    }

    fn drag(&mut self, mx: f32, my: f32) {
        if self.dragging {
            self.position.x = mx + self.drag_offset.x;
            self.position.y = my + self.drag_offset.y;
        }
    }
}

impl Mover {
    fn new(m: f32, x: f32, y: f32) -> Self {
        let mass = m;
        let position = pt2(x, y);
        let velocity = vec2(1.0, 0.0);
        let acceleration = vec2(0.0, 0.0);
        Mover {
            position,
            velocity,
            acceleration,
            mass,
        }
    }

    fn apply_force(&mut self, force: Vec2) {
        let f = force / self.mass;
        self.acceleration += f;
    }

    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position += self.velocity;
        self.acceleration *= 0.0;
    }

    fn display(&self, draw: &DrawHolder) {
        draw.ellipse()
            .xy(self.position)
            .w_h(self.mass * 16.0, self.mass * 16.0)
            .rgba(0.0, 0.0, 0.0, 0.5)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }
}

fn model(app: &App) -> Model {
    let rect = Rect::from_w_h(640.0, 360.0);
    let _window = app
        .new_window()
        .size(rect.w() as u32, rect.h() as u32)
        .event(event)
        .view(view)
        .build();

    let movers = (0..90)
        .map(|_| {
            Mover::new(
                random_range(0.1f32, 2.0),
                random_range(rect.left(), rect.right()),
                random_range(rect.top(), rect.bottom()),
            )
        })
        .collect();

    let attractor = Attractor::new(rect);

    Model { movers, attractor }
}

fn event(app: &App, m: &mut Model, event: WindowEvent) {
    match event {
        MousePressed(_button) => {
            m.attractor.clicked(app.mouse.x, app.mouse.y);
        }
        MouseReleased(_buttom) => {
            m.attractor.stop_dragging();
        }
        _other => (),
    }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.attractor.drag(app.mouse.x, app.mouse.y);
    m.attractor.hover(app.mouse.x, app.mouse.y);
    for i in 0..m.movers.len() {
        let force = m.attractor.attract(&m.movers[i]);
        m.movers[i].apply_force(force);
        m.movers[i].update();
    }
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.attractor.display(&draw);

    // Draw movers
    for mover in &m.movers {
        mover.display(&draw);
    }



}
