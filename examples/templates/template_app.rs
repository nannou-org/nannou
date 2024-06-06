use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: Entity,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build();
    Model { _window }
}

fn update(_app: &App, _model: &mut Model) {}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(PLUM);
    draw.ellipse().color(STEEL_BLUE);

}
