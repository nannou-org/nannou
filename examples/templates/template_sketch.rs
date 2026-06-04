use nannou::prelude::*;

fn main() {
    nannou::sketch(view).run()
}

fn view(app: &App) {
    let draw = app.draw();
    draw.background().color(PLUM);
    draw.ellipse().color(STEEL_BLUE);
}
