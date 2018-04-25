// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 3-9: Wave_A
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
    angle: f32,
}

fn model(app: &App) -> Model {
    let window = app.new_window().with_dimensions(400,400).build().unwrap();
    let angle = 0.0;
    Model { window, angle }
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
            model.angle += 0.02;
        }
        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    //app.main_window().set_title("Wave A");

    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(1.0, 1.0, 1.0);

    let y = 100.0 * model.angle.sin();

    draw.line()
        .start(Point2::new(0.0,0.0))
        .end(Point2::new(0.0,y))
        .rgb(0.4,0.4,0.4);
    draw.ellipse()
        .x_y(0.0, y)
        .w_h(16.0, 16.0)
        .rgb(0.4, 0.4, 0.4);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
