use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: &Frame) {
    // Prepare to draw.
    let draw = app.draw();

    let win = app.window_rect();

    let color_select = map_range(app.mouse.y, win.top(), win.bottom(), 0.0, 5.0) as i32;

    let bg_color = match color_select {
        0 => RED,
        1 => ORANGE,
        2 => YELLOW,
        3 => GREEN,
        4 => BLUE,
        _ => BLACK,
    };

    draw.background().color(bg_color);

    if app.mouse.x < 0.0 {
        draw.ellipse().color(STEELBLUE);
    } else {
        draw.ellipse().color(SEAGREEN);
    }

    // Draw to the window frame.
    draw.to_frame(app, &frame).unwrap();
}
