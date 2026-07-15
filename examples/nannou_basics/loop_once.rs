//! `loop_once` - draw a single frame, then hold it on screen.
//!
//! `view` runs exactly once: the composition is drawn and then frozen. The window
//! idles (no CPU) and stays closable and resizable, but nothing ever redraws -
//! waiting or moving the mouse changes nothing. This is the modern equivalent of
//! the old `LoopMode::loop_once()`, ideal for static, sketch-based art.
//!
//! Swap `loop_once()` for `loop_ntimes(n)` to advance `n` frames first, or drop it
//! entirely to animate continuously.

use nannou::prelude::*;

fn main() {
    nannou::sketch(view).size(600, 600).loop_once().run();
}

fn view(app: &App) {
    let draw = app.draw();
    draw.background().color(SNOW);

    // A static spiral of circles, drawn once and then held on screen.
    let win = app.window_rect();
    let n = 240;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let angle = t * PI * 24.0;
        let radius = t * win.w() * 0.45;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        draw.ellipse()
            .x_y(x, y)
            .radius(6.0 * (1.0 - t) + 1.0)
            .hsv(t, 0.7, 0.9);
    }
}
