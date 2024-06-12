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
    let a = app.new_window().focused(focus_a).title("window a").build();
    let b = app.new_window().focused(focus_b).title("window b").build();
    let c = app.new_window().focused(focus_c).title("window c").build();
    Model { a, b, c }
}

fn focus_a(app: &App, model: &mut Model) {
    info!("focusing window a");
}

fn focus_b(app: &App, model: &mut Model) {
    info!("focusing window b");
}

fn focus_c(app: &App, model: &mut Model) {
    info!("focusing window c");
}

fn update(_app: &App, _model: &mut Model) {}

fn view(app: &App, model: &Model, window: Entity) {
    let draw = app.draw_for_window(window);
    match window {
        id if id == model.a => {
            draw.tri().color(RED);
            draw.background().color(INDIAN_RED)
        }
        id if id == model.b => {
            draw.tri().color(GREEN);
            draw.background().color(LIGHT_GREEN)
        }
        id if id == model.c => {
            draw.tri().color(BLUE);
            draw.background().color(CORNFLOWER_BLUE)
        }
        _ => panic!("unexpected window entity"),
    };
}
