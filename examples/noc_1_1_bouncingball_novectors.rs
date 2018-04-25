// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example x-x: Bouncing Ball, no vectors
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
    x: f32,
    y: f32,
    x_speed: f32,
    y_speed: f32,
}

fn model(app: &App) -> Model {
    let x = 100.0;
    let y = 100.0;
    let x_speed = 2.5;
    let y_speed = 2.0;

    let window = app.new_window().with_dimensions(800, 200).build().unwrap();
    Model {
        window,
        x,
        y,
        x_speed,
        y_speed,
    }
}

fn event(app: &App, mut model: Model, event: Event) -> Model {
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
            // Add the current speed to the position
            model.x = model.x + model.x_speed;
            model.y = model.y + model.y_speed;

            let win_rect = app.window.rect();
            
            if (model.x > win_rect.right()) || (model.x < win_rect.left()) {
                model.x_speed = model.x_speed * -1.0;
            }
            if (model.y > win_rect.top()) || (model.y < win_rect.bottom()) {
                model.y_speed = model.y_speed * -1.0;
            }
        }
        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    app.main_window().set_title("noc_1_1 bouncing ball no vectors");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    draw.ellipse()
        .x_y(model.x, model.y)
        .w_h(50.0, 50.0)
        .rgba(0.5, 0.5, 0.5, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
