//! A simple example demonstrating how to produce path events from text.
//!
//! While drawing text via the `Path` API may not be the most efficient approach, it allows for
//! interesting creative applications.

use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    // Begin drawing.
    let draw = app.draw();
    draw.background().color(WHITE);

    let win_rect = app.main_window().rect();
    draw.rect()
        .hsla(0.0, 0.0, 0.5, 0.5)
        .y(win_rect.bottom() * 0.5)
        .w_h(win_rect.w(), win_rect.top());

    // Draw the text.
    let text = text("create\nwith\nnannou").font_size(128).build(win_rect);

    // Draw rects behind the lines.
    for line_rect in text.line_rects() {
        let a = map_range(app.mouse().x, win_rect.left(), win_rect.right(), 0.0, 1.0);
        draw.rect().xy(line_rect.xy()).wh(line_rect.wh()).hsla(
            -line_rect.y() / win_rect.top(),
            1.0,
            0.5,
            a,
        );
    }

    // Draw rects behind the glyphs.
    for (_glyph, rect) in text.glyphs() {
        let a = map_range(app.mouse().x, win_rect.bottom(), win_rect.top(), 0.0, 1.0);
        draw.rect().xy(rect.xy()).wh(rect.wh()).hsla(
            (rect.x() + rect.y()) / win_rect.w(),
            1.0,
            0.5,
            a,
        );
    }

    draw.path().fill().color(BLACK).events(text.path_events());



}
