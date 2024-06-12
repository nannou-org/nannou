use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    // Begin drawing
    let draw = app.draw();

    draw.tri().width(100.0).points().color(RED);
}
