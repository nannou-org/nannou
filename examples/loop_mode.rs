extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model;

fn model(app: &App) -> Model {
    // Start in `Wait` mode. In other words, don't keep looping, just wait for events.
    app.set_loop_mode(LoopMode::wait(3));
    // Set a window title.
    app.main_window().set_title("`LoopMode` Demonstration");
    Model
}

fn event(app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent { simple: Some(event), .. } => match event {
            KeyPressed(_) => {
                match app.loop_mode() {
                    LoopMode::Rate { .. } => app.set_loop_mode(LoopMode::wait(3)),
                    LoopMode::Wait { .. } => app.set_loop_mode(LoopMode::rate_fps(60.0)),
                }
                println!("Loop mode switched to: {:?}", app.loop_mode());
            },
            _ => (),
        },
        Event::Update(update) => {
            println!("{:?}", update);
        },
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    frame.clear_all(DARK_CHARCOAL);
    frame
}
