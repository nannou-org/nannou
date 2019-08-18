use nannou::prelude::*;
use std::time::Duration;

mod screenshot;

use screenshot::Shots;

struct Model {
    screenshot: Shots,
}

fn main() {
    nannou::app(model).exit(exit).run();
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .with_dimensions(1024, 768)
        .view(view)
        .event(window_event)
        .build()
        .unwrap();
    let screenshot = screenshot::new(app, window_id);
    Model { screenshot }
}

fn view(app: &App, model: &Model, frame: &Frame) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(CORNFLOWERBLUE);

    // Draw a purple triangle in the top left half of the window.
    let win = app.window_rect();
    draw.tri()
        .points(win.bottom_left(), win.top_left(), win.top_right())
        .color(VIOLET);

    // Draw an ellipse to follow the mouse.
    let t = app.time;
    draw.ellipse()
        .x_y(app.mouse.x * t.cos(), app.mouse.y)
        .radius(win.w() * 0.125 * t.sin())
        .color(RED);

    // Draw a line!
    draw.line()
        .weight(10.0 + (t.sin() * 0.5 + 0.5) * 90.0)
        .caps_round()
        .color(PALEGOLDENROD)
        .points(win.top_left() * t.sin(), win.bottom_right() * t.cos());

    // Draw a quad that follows the inverse of the ellipse.
    draw.quad()
        .x_y(-app.mouse.x, app.mouse.y)
        .color(DARKGREEN)
        .rotate(t);

    // Draw a rect that follows a different inverse of the ellipse.
    draw.rect()
        .x_y(app.mouse.y, app.mouse.x)
        .w(app.mouse.x * 0.25)
        .hsv(t, 1.0, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // This only captures if take() is called
    model.screenshot.capture(&frame);
}

fn window_event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(key) => {
            if let Key::S = key {
                // Adds a screenshot to the queue to be taken
                model.screenshot.take();
            }
        }
        KeyReleased(_key) => {}
        MouseMoved(_pos) => {}
        MousePressed(_button) => {}
        MouseReleased(_button) => {}
        MouseEntered => {}
        MouseExited => {}
        MouseWheel(_amount, _phase) => {}
        Moved(_pos) => {}
        Resized(_size) => {}
        Touch(_touch) => {}
        TouchPressure(_pressure) => {}
        HoveredFile(_path) => {}
        DroppedFile(_path) => {}
        HoveredFileCancelled => {}
        Focused => {}
        Unfocused => {}
        Closed => {}
    }
}

fn exit(_: &App, model: Model) {
    // If you are getting an Access error then you
    // might need to raise the wait time
    model.screenshot.flush(Duration::from_secs(3));
}
