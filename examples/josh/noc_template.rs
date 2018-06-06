// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example x-x:
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
    param1: i32,
    param2: i32,
    param3: f32,
    vec_values: Vec<f32>,
}

fn model(app: &App) -> Model {
    let param1 = 20;
    let param2 = 0;
    let param3 = 0.1;

    // Note you can decalre and pack a vector with random values like this in rust
    let vec_values = (0..15).map(|_| 0.0).collect();

    let window = app.new_window().with_dimensions(640, 360).build().unwrap();
    Model {
        window,
        param1,
        param2,
        param3,
        vec_values,
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
        Event::Update(_dt) => {}
        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);
    draw.rect()
        .x_y(0.0, 0.0)
        .w_h(50.0, 50.0)
        .rgba(1.0, 0.5, 0.3, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
