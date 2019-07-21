use nannou::prelude::*;

fn main() {
    nannou::sketch(view);
}

fn view(app: &App, frame: &Frame) {
    app.main_window().set_inner_size_pixels(800, 400);

    // Begin drawing
    let draw = app.draw();

    draw.background().color(BLACK);
    let win_rect = app.window_rect();

    let step_x = (app.mouse.x - win_rect.left()).max(5.0);
    let step_y = (win_rect.top() - app.mouse.y).max(5.0);

    let size = vec2(step_x, step_y);
    let r = nannou::geom::Rect::from_wh(size)
        .align_left_of(win_rect)
        .align_top_of(win_rect);
    let mut grid_y = 0.0;
    while grid_y < win_rect.h() {
        let mut grid_x = 0.0;
        while grid_x < win_rect.w() {
            let r = r.shift_x(grid_x).shift_y(-grid_y);
            let hue = grid_x / win_rect.w();
            let saturation = 1.0 - (grid_y / win_rect.h());
            draw.rect().xy(r.xy()).wh(r.wh()).hsl(hue, saturation, 0.5);
            grid_x += step_x;
        }
        grid_y += step_y;
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
