//! A simple example demonstrating the `draw.text("foo")` API.
//!
//! If you're looking for more control over the text path, checkout `simple_text_path.rs`.

use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    // Begin drawing.
    let draw = app.draw();
    draw.background().color(WHITE);

    // We'll align to the window dimensions, but padded slightly.
    let win_rect = app.main_window().rect().pad(20.0);

    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n\nResize the window to test dynamic layout.";

    //                         L     o     r     e     m           i    p    s    u    m
    let glyph_colors = vec![BLUE, BLUE, BLUE, BLUE, BLUE, BLACK, RED, RED, RED, RED, RED];

    draw.text(text)
        .color(BLACK)
        .glyph_colors(glyph_colors)
        .font_size(24)
        .wh(win_rect.wh());
}
