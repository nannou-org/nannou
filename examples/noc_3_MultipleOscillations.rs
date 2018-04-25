// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-x: Multiple Oscillations 
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
    angle1: f32,
    a_velocity1: f32,
    amplitude1: f32,
    angle2: f32,
    a_velocity2: f32,
    amplitude2: f32,
}

fn model(app: &App) -> Model {
    let mut angle1 = 0.0;
    let a_velocity1 = 0.01;
    let amplitude1 = 300.0;
    
    let mut angle2 = 0.0;
    let a_velocity2 = 0.3;
    let amplitude2 = 10.0;

    let window = app.new_window().with_dimensions(640, 360).build().unwrap();
    Model {
        window,
        angle1,
        a_velocity1,
        amplitude1,
        angle2,
        a_velocity2,
        amplitude2,
    }
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
            model.angle1 += model.a_velocity1;
            model.angle2 += model.a_velocity2;
        }
        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("Multiple Oscillations");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    let mut x = 0.0;
    x += model.amplitude1 * model.angle1.cos();
    x += model.amplitude2 * model.angle2.sin();

    draw.ellipse()
        .x_y(x as f32, 0.0)
        .w_h(20.0, 20.0)
        .rgba(0.7, 0.7, 0.7, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
