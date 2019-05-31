use nannou::prelude::*;
use std::convert::TryInto;

mod screenshot;

use screenshot::FrameLock;

// These must be smaller then your actual screen
const IMAGE_DIMS: (usize, usize) = (1366, 600);

struct Model {
    screenshot: FrameLock,
}

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .with_dimensions(
            IMAGE_DIMS.0.try_into().unwrap(),
            IMAGE_DIMS.1.try_into().unwrap(),
        )
        .view(view)
        .event(window_event)
        .build()
        .unwrap();
    let screenshot = screenshot::new(app, IMAGE_DIMS);
    Model {
        screenshot,
    }
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    model.screenshot.take(&frame);
    // Begin drawing
    let draw = app.draw();

    // Clear the background to blue.
    draw.background().color(BLUE);

    // Draw a purple triangle in the top left half of the window.
    let win = app.window_rect();
    draw.tri()
        .points(win.bottom_left(), win.top_left(), win.top_right())
        .color(DARK_PURPLE);

    // Draw an ellipse to follow the mouse.
    let t = app.time;
    draw.ellipse()
        .x_y(app.mouse.x * t.cos(), app.mouse.y)
        .radius(win.w() * 0.125 * t.sin())
        .color(RED);

    // Draw a line!
    draw.line()
        .start(win.top_left() * t.sin())
        .end(win.bottom_right() * t.cos())
        .thickness(win.h() / (50.0 * t.sin()))
        .caps_round()
        .color(LIGHT_YELLOW);

    // Draw a quad that follows the inverse of the ellipse.
    draw.quad()
        .x_y(-app.mouse.x, app.mouse.y)
        .color(DARK_GREEN)
        .rotate(t);

    // Draw a rect that follows a different inverse of the ellipse.
    draw.rect()
        .x_y(app.mouse.y, app.mouse.x)
        .w(app.mouse.x * 0.25)
        .hsv(t, 1.0, 1.0);

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    model.screenshot.copy_frame(&frame);


    // Return the drawn frame.
    frame
}

fn window_event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(key) => {
            if let Key::S = key {
                unimplemented!();
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
