use nannou::prelude::*;

mod ball;
use crate::ball::Ball;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    ball: Ball,
}

fn model(_app: &App) -> Model {
    // Create a new ball with a color
    let ball = Ball::new(GREEN);
    // Construct and return the model with our initialised ball
    Model { ball }
}

// By default, `update` is called right before `view` is called each frame.
fn update(app: &App, model: &mut Model, _update: Update) {
    model.ball.position = pt2(app.mouse.x, app.mouse.y);
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing.
    let draw = app.draw();
    // Draw dark gray for the background
    draw.background().color(DIMGRAY);
    // Draw our ball
    model.ball.display(&draw);
    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
