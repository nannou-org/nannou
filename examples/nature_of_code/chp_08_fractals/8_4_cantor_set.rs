// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 8-4: Cantor Set
// Renders a simple fractal, the Cantor Set
use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model;

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::loop_once());
    let _window = app
        .new_window()
        .with_dimensions(800, 200)
        .view(view)
        .build()
        .unwrap();
    Model
}

fn view(app: &App, _model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().color(WHITE);

    let win = app.window_rect();
    cantor(&draw, 0.0, win.top(), 730.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}

// Recursive function
fn cantor(draw: &app::Draw, x: f32, mut y: f32, len: f32) {
    let h = 30.0;

    // recursive exit condition
    if len >= 1.0 {
        // Draw line (as rectangle to make it easier to see)
        draw.rect()
            .x_y(x, y - h / 6.0)
            .w_h(len, h / 3.0)
            .color(BLACK);

        // Go down to next y position
        y -= h;
        // Draw 2 more lines 1/3rd the length (without the middle section)
        let length = len / 3.0;
        cantor(&draw, x - length, y, length);
        cantor(&draw, (x + len * 2.0 / 3.0) - length, y, length);
    }
}
