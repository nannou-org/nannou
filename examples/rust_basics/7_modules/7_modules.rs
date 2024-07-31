use nannou::prelude::*;

use crate::ball::Ball;

mod ball;

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
fn update(app: &App, model: &mut Model) {
    model.ball.position = pt2(app.mouse().x, app.mouse().y);
}

fn view(app: &App, model: &Model, _window: Entity) {
    // Begin drawing.
    let draw = app.draw();
    // Draw dark gray for the background
    draw.background().color(DIM_GRAY);
    // Draw our ball
    model.ball.display(&draw);
}
