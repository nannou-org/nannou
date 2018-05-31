// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-9: Wave_C
extern crate nannou;

use nannou::prelude::*;
use nannou::math::map_range;
use nannou::rand::random;
use nannou::color::Rgba;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
    start_angle: f32,
    angle_vel: f32,
}

fn model(app: &App) -> Model {
    let window = app.new_window().with_dimensions(200,200).build().unwrap();
    let start_angle = 0.0;
    let angle_vel = 0.4;
    Model { window, start_angle, angle_vel}
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
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
            model.start_angle += 0.015;
        }
        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("Wave C");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    let mut angle = model.start_angle;
    let rect = app.window_rect();
    let mut x = rect.left();
    while x <= rect.right() {
        let y = map_range(angle.sin(), -1.0, 1.0, rect.top(), rect.bottom()); 
        draw.ellipse()
            .x_y(x as f32, y)
            .w_h(48.0, 48.0)
            .rgba(0.0, 0.0, 0.0, 0.5);

        angle += model.angle_vel;
        x += 24.0;
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
