use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    a: WindowId,
    b: WindowId,
    c: WindowId,
}

fn model(app: &App) -> Model {
    let a = app
        .new_window()
        .title("window a")
        .event(event_a)
        .build()
        .unwrap();
    let b = app
        .new_window()
        .title("window b")
        .event(event_b)
        .build()
        .unwrap();
    let c = app
        .new_window()
        .title("window c")
        .event(event_c)
        .build()
        .unwrap();
    Model { a, b, c }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn event_a(_app: &App, _model: &mut Model, event: WindowEvent) {
    println!("window a: {:?}", event);
}

fn event_b(_app: &App, _model: &mut Model, event: WindowEvent) {
    println!("window b: {:?}", event);
}

fn event_c(_app: &App, _model: &mut Model, event: WindowEvent) {
    println!("window c: {:?}", event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    match frame.window_id() {
        id if id == model.a => {
            draw.background().color(INDIANRED);
            draw.ellipse().color(LIGHTGREEN);
        }
        id if id == model.b => {
            draw.background().color(LIGHTGREEN);
            draw.tri().color(CORNFLOWERBLUE);
        }
        id if id == model.c => {
            draw.background().color(CORNFLOWERBLUE);
            draw.rect().color(INDIANRED);
        }
        _ => (),
    }

    draw.to_frame(app, &frame).unwrap();
}
