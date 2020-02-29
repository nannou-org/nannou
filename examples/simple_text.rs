//! A simple example demonstrating the `draw.text("foo")` API.
//!
//! If you're looking for more control over the text path, checkout `simple_text_path.rs`.

use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: Frame) {
    // Begin drawing.
    let draw = app.draw();
    draw.background().color(WHITE);

    // We'll align to the window dimensions, but padded slightly.
    let win_rect = app.main_window().rect().pad(20.0);

    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n\nResize the window to test dynamic layout.";

    draw.text(text).color(BLACK).font_size(24).wh(win_rect.wh());

    draw.to_frame(app, &frame).unwrap();
}
