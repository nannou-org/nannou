// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-16: Exercise Springs
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// Spring Type describes an anchor point that can connect to "Bob" objects via a spring
// Thank you: http://www.myphysicslab.com/spring2d.html
struct Spring {
    // Rest length and spring constant
    len: f32,
    k: f32,
}

impl Spring {
    fn new(l: f32) -> Self {
        Spring { len: l, k: 0.2 }
    }

    // Calculate spring force
    fn update(&self, a: &mut Bob, b: &mut Bob) {
        // Vector pointing from anchor to bob position
        let mut force = a.position - b.position;
        // What is the distance
        let d = force.magnitude();
        // Stretch is difference between current distance and rest length
        let stretch = d - self.len;

        // Calculate force according to Hooke's Law
        // F = k * stretch
        force = force.normalize();
        force *= -1.0 * self.k * stretch;
        a.apply_force(force);
        force *= -1.0;
        b.apply_force(force);
    }

    fn display(&self, draw: &app::Draw, a: &Bob, b: &Bob) {
        draw.line()
            .start(a.position)
            .end(b.position)
            .color(BLACK)
            .stroke_weight(2.0);
    }
}

struct Bob {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
    mass: f32,
    damping: f32,
    drag_offset: Vector2,
    dragging: bool,
}

impl Bob {
    fn new(x: f32, y: f32) -> Self {
        Bob {
            position: pt2(x, y),
            velocity: vec2(0.0, 0.0),
            acceleration: vec2(0.0, 0.0),
            mass: 12.0,
            damping: 0.95, // Arbitrary damping to simulate friction / drag
            drag_offset: vec2(0.0, 0.0),
            dragging: false,
        }
    }

    // Standard Euler integration
    fn update(&mut self) {
        self.velocity += self.acceleration;
        self.velocity *= self.damping;
        self.position += self.velocity;
        self.acceleration *= 0.0;
    }

    // Newton's law: F = M * A
    fn apply_force(&mut self, force: Vector2) {
        let f = force / self.mass;
        self.acceleration += f;
    }

    fn display(&self, draw: &app::Draw) {
        let c = if self.dragging { GREY } else { DARKGREY };
        draw.ellipse()
            .xy(self.position)
            .w_h(self.mass * 2.0, self.mass * 2.0)
            .color(c)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }

    // The methods below are for mouse interaction

    // This checks to see if we clicked on the mover
    fn clicked(&mut self, mx: f32, my: f32) {
        let d = pt2(mx, my).distance(self.position);
        if d < self.mass {
            self.dragging = true;
            self.drag_offset.x = self.position.x - mx;
            self.drag_offset.y = self.position.y - my;
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

struct Model {
    b1: Bob,
    b2: Bob,
    b3: Bob,
    s1: Spring,
    s2: Spring,
    s3: Spring,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(640, 360)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build()
        .unwrap();

    // Create objects at starting position
    // Note third argument in Spring constructor is "rest length"
    let win = app.window_rect();

    Model {
        b1: Bob::new(0.0, win.top() - 100.0),
        b2: Bob::new(0.0, win.top() - 200.0),
        b3: Bob::new(0.0, win.top() - 300.0),
        s1: Spring::new(100.0),
        s2: Spring::new(100.0),
        s3: Spring::new(100.0),
    }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    m.s1.update(&mut m.b1, &mut m.b2);
    m.s2.update(&mut m.b2, &mut m.b3);
    m.s3.update(&mut m.b1, &mut m.b3);

    m.b1.update();
    m.b2.update();
    m.b3.update();
    m.b1.drag(app.mouse.x, app.mouse.y);
}

fn view(app: &App, m: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.s1.display(&draw, &m.b1, &m.b2);
    m.s2.display(&draw, &m.b2, &m.b3);
    m.s3.display(&draw, &m.b1, &m.b3);

    m.b1.display(&draw);
    m.b2.display(&draw);
    m.b3.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn mouse_pressed(app: &App, m: &mut Model, _button: MouseButton) {
    m.b1.clicked(app.mouse.x, app.mouse.y);
}

fn mouse_released(_app: &App, m: &mut Model, _button: MouseButton) {
    m.b1.stop_dragging();
}
