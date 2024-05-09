// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-11: Spring
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// Spring Type describes an anchor point that can connect to "Bob" objects via a spring
// Thank you: http://www.myphysicslab.com/spring2d.html
struct Spring {
    // position
    anchor: Point2,
    // Rest length and spring constant
    len: f32,
    k: f32,
}

impl Spring {
    fn new(x: f32, y: f32, l: f32) -> Self {
        Spring {
            anchor: pt2(x, y),
            len: l,
            k: 0.2,
        }
    }

    // Calculate spring force
    fn connect(&self, b: &mut Bob) {
        // Vector pointing from anchor to bob position
        let mut force = b.position - self.anchor;
        // What is the distance
        let d = force.length();
        // Stretch is difference between current distance and rest length
        let stretch = d - self.len;

        // Calculate force according to Hooke's Law
        // F = k * stretch
        force = force.normalize();
        force *= -1.0 * self.k * stretch;
        b.apply_force(force);
    }

    // Constrain the distance between bob and anchor between min and max
    fn constrain_length(&self, b: &mut Bob, min_len: f32, max_len: f32) {
        let mut dir = b.position - self.anchor;
        let d = dir.length();
        // Is it too short?
        if d < min_len {
            dir = dir.normalize();
            dir *= min_len;
            // Reset position and stop from moving (not realistic physics)
            b.position = self.anchor + dir;
            b.velocity *= 0.0;
        }
        // Is it too long?
        else if d > max_len {
            dir = dir.normalize();
            dir *= max_len;
            // Reset position and stop from moving (not realistic physics)
            b.position = self.anchor + dir;
            b.velocity *= 0.0;
        }
    }

    fn display(&self, draw: &DrawHolder) {
        draw.rect()
            .xy(self.anchor)
            .w_h(10.0, 10.0)
            .color(GREY)
            .stroke(BLACK)
            .stroke_weight(2.0);
    }

    fn display_line(&self, draw: &DrawHolder, bob: &Bob) {
        draw.line()
            .start(bob.position)
            .end(self.anchor)
            .color(BLACK)
            .stroke_weight(2.0);
    }
}

struct Bob {
    position: Point2,
    velocity: Vec2,
    acceleration: Vec2,
    mass: f32,
    damping: f32,
    drag_offset: Vec2,
    dragging: bool,
}

impl Bob {
    fn new(x: f32, y: f32) -> Self {
        Bob {
            position: pt2(x, y),
            velocity: vec2(0.0, 0.0),
            acceleration: vec2(0.0, 0.0),
            mass: 24.0,
            damping: 0.98, // Arbitrary damping to simulate friction / drag
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
    fn apply_force(&mut self, force: Vec2) {
        let f = force / self.mass;
        self.acceleration += f;
    }

    fn display(&self, draw: &DrawHolder) {
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
    bob: Bob,
    spring: Spring,
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
        bob: Bob::new(0.0, win.top() - 100.0),
        spring: Spring::new(0.0, win.top() - 10.0, 100.0),
    }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    // Apply a gravity force to the bob
    let gravity = pt2(0.0, -2.0);
    m.bob.apply_force(gravity);

    // Connect the bob to the spring (this calculates the force)
    m.spring.connect(&mut m.bob);
    // Constrain spring distance between min and max
    m.spring.constrain_length(&mut m.bob, 30.0, 200.0);

    // Update bob
    m.bob.update();
    // if it's being dragged
    m.bob.drag(app.mouse.x, app.mouse.y);
}

fn view(app: &App, m: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    m.spring.display_line(&draw, &m.bob); // Draw a line between spring and bob
    m.bob.display(&draw);
    m.spring.display(&draw);



}

fn mouse_pressed(app: &App, m: &mut Model, _button: MouseButton) {
    m.bob.clicked(app.mouse.x, app.mouse.y);
}

fn mouse_released(_app: &App, m: &mut Model, _button: MouseButton) {
    m.bob.stop_dragging();
}
