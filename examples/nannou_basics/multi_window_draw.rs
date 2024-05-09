use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    a: Entity,
    b: Entity,
    c: Entity,
}

fn model(app: &App) -> Model {
    let a = app
        .new_window()
        .title("window a")
        .focused(event_a)
        .build();
    let b = app
        .new_window()
        .title("window b")
        .focused(event_b)
        .build();
    let c = app
        .new_window()
        .title("window c")
        .focused(event_c)
        .build();
    Model { a, b, c }
}

fn update(_app: &App, _model: &mut Model) {

}

fn event_a(_app: &App, _model: &mut Model) {
    println!("window a");
}

fn event_b(_app: &App, _model: &mut Model) {
    println!("window b");
}

fn event_c(_app: &App, _model: &mut Model) {
    println!("window c");
}

fn view(app: &App, model: &Model, window: Entity) {
    let draw = app.draw();
    match window {
        id if id == model.a => {
            draw.background().color(INDIAN_RED);
            draw.ellipse().color(LIGHT_GREEN);
        }
        id if id == model.b => {
            draw.background().color(LIGHT_GREEN);
            draw.tri().color(CORNFLOWER_BLUE);
        }
        id if id == model.c => {
            draw.background().color(CORNFLOWER_BLUE);
            draw.rect().color(INDIAN_RED);
        }
        _ => (),
    }


}
