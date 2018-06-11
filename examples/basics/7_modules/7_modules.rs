extern crate nannou;

use nannou::prelude::*;

mod ball;
use ball::Ball;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    ball: Ball,
}

fn model(_app: &App) -> Model {
    // Create a new ball with a color
    let ball = Ball::new(Rgba::new(0.0, 1.0, 0.5, 1.0));

    // Construct and return the model with our initialised ball
    Model { ball }
}

fn event(app: &App, mut model: Model, event: Event) -> Model {
    // update gets called just before view every frame
    if let Event::Update(_update) = event {
        model.ball.position = pt2(app.mouse.x, app.mouse.y);
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    // Draw dark gray for the background
    draw.background().rgb(0.02, 0.02, 0.02);
    // Draw our ball
    model.ball.display(&draw);
    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();
    // Return the drawn frame.
    frame
}
