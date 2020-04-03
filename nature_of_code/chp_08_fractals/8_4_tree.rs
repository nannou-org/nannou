// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Recursive Tree
// Renders a simple tree-like structure via recursion
// Branching angle calculated as a function of horizontal mouse position
// Example 8-4: Tree

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    theta: f32,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size(300, 200).view(view).build().unwrap();
    Model { theta: 0.0 }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let win = app.window_rect();
    model.theta = map_range(app.mouse.x, win.left(), win.right(), 0.0, PI / 2.0);
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);

    let win = app.window_rect();
    let draw = app.draw().x_y(0.0, win.bottom());

    branch(&draw, 60.0, model.theta);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn branch(draw: &Draw, len: f32, theta: f32) {
    let mut length = len;
    // Each branch will be 2/3rds the size of the previous one
    let sw = map_range(length, 2.0, 120.0, 1.0, 10.0);

    draw.line()
        .start(pt2(0.0, 0.0))
        .end(pt2(0.0, length))
        .weight(sw)
        .color(BLACK);
    // Move to the end of that line
    let draw = draw.x_y(0.0, length);

    length *= 0.66;

    // All recursive functions must have an exit condition!!!!
    // Here, ours is when the length of the branch is 2 pixels or less
    if len > 2.0 {
        let draw2 = draw.rotate(theta); // Save the current state of transformation (i.e. where are we now) and Rotate by theta
        branch(&draw2, length, theta); // Ok, now call myself to draw two new branches!!

        // Repeat the same thing, only branch off to the "left" this time!
        let draw3 = draw.rotate(-theta);
        branch(&draw3, length, theta);
    }
}
