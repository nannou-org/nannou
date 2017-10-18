extern crate nannou;

use nannou::{App, Event, Frame, LoopMode};
use nannou::event::SimpleWindowEvent::*;

fn main() {
    nannou::run(model, update, draw);
}

struct Model {
    window: nannou::window::Id,
}

fn model(app: &App) -> Model {
    // Start in `Wait` mode. In other words, don't keep looping, just wait for events.
    app.set_loop_mode(LoopMode::wait(3));
    let window = app.new_window().build().unwrap();
    Model { window }
}

fn update(app: &App, model: Model, event: Event) -> Model {
    println!("{:?}", event);
    match event {
        Event::WindowEvent(_id, event) => match event.simple {
            KeyPressed(_) => match app.loop_mode() {
                LoopMode::Rate { .. } => app.set_loop_mode(LoopMode::wait(3)),
                LoopMode::Wait { .. } => app.set_loop_mode(LoopMode::rate_fps(60.0)),
            },
            _ => (),
        },
        Event::Update(_update) => {
        },
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn draw(_app: &App, model: &Model, frame: Frame) -> Frame {
    frame.window(model.window).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);
    frame
}
