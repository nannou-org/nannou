// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Exercise 3-04: Spiral
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
    r: f32,
    theta: f32,
}

fn model(app: &App) -> Model {
    let mut r = 0.0;
    let mut theta = 0.0;

    let window = app.new_window().with_dimensions(640, 360).build().unwrap();
    Model {
        window,
        r,
        theta,
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
            // Increment the angle
            model.theta += 0.01;
            // Increment the radius
            model.r += 0.05;
        }
        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("Spiral");

    // Begin drawing
    let draw = app.draw();
    //draw.background().rgb(1.0, 1.0, 1.0);

    let x = model.r * model.theta.cos();
    let y = model.r * model.theta.sin();

    // Draw an ellipse at x,y
    draw.ellipse()
        .x_y(x, y)
        .w_h(16.0, 16.0)
        .rgba(0.0, 0.0, 0.0, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
