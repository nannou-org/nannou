extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::view(view);
}

fn view(app: &App, frame: Frame) -> Frame {
    app.main_window().set_inner_size_pixels(800, 400);

    // Begin drawing
    let draw = app.draw();

    draw.background().color(BLACK);
    let win_rect = app.window_rect();

    let step_x = (app.mouse.x - win_rect.left()).max(1.0);
    let step_y = (win_rect.top() - app.mouse.y).max(1.0);

    let size = vec2(step_x, step_y);
    let r = nannou::geom::Rect::from_wh(size)
        .align_left_of(win_rect)
        .align_top_of(win_rect);
    // for grid_y in (0..).map(|i| i as f32 * step_y).take_while(|&y| y < app.window_rect().h()) {
    //     for grid_x in (0..).map(|i| i as f32 * step_x).take_while(|&x| x < app.window_rect().w()) {
    let mut grid_y = 0.0;
    while grid_y < win_rect.h() {
        let mut grid_x = 0.0;
        while grid_x < win_rect.w() {
            let r = r.shift_x(grid_x).shift_y(-grid_y);
            let hue = grid_x / win_rect.w();
            let saturation = 1.0 - (grid_y / win_rect.h());
            draw.rect().xy(r.xy()).wh(r.wh()).hsl(hue, saturation, 0.5);

            // let w = step_x;
            // let h = step_y;
            // let half_w = w * 0.5;
            // let half_h = h * 0.5;
            // let x = half_w + grid_x - app.window_rect().right();
            // let y = app.window_rect().top() - half_h - grid_y;
            // draw.rect()
            //     .x_y(x, y)
            //     .w_h(w, h)
            //     .hsl(
            //         (grid_x / app.window_rect().w()) * 360.0,
            //         //0.9,
            //         1.0 - grid_y / app.window_rect().h(),
            //         0.5,
            //     );
            grid_x += step_x;
        }
        grid_y += step_y;
    }

    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
