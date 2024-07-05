use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    let draw = app.draw();
    draw.background().color(WHITE);

    let win = app.window_rect();
    let num_bars = 20;
    let bar_width = win.w() / num_bars as f32;

    for i in 0..num_bars {
        let x = map_range(i, 0, num_bars, win.left(), win.right());
        let y = map_range(i, 0, num_bars, win.top(), win.bottom());

        draw.rect()
            .x_y(x, y)
            .w_h(bar_width, win.h() / num_bars as f32)
            .color(BLACK);
    }
}
