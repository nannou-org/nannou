//! A simple example demonstrating the behaviour of the three `LoopMode` variants supported by
//! nannou.
//!
//! The `LoopMode` determines how the nannou application loop is driven.
//!
//! See the `LoopMode` docs for more details:
//!
//! https://docs.rs/nannou/latest/nannou/app/enum.LoopMode.html

extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

struct Model;

fn model(app: &App) -> Model {
    // Start in `Wait` mode. In other words, don't keep looping, just wait for events.
    app.set_loop_mode(LoopMode::wait(3));
    // Set a window title.
    let title = format!("`LoopMode` Demonstration - `{:?}`", app.loop_mode());
    app.main_window().set_title(&title);
    Model
}

fn event(app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => match event {
            KeyPressed(_) => {
                match app.loop_mode() {
                    LoopMode::Wait { .. } => app.set_loop_mode(LoopMode::refresh_sync()),
                    LoopMode::RefreshSync { .. } => app.set_loop_mode(LoopMode::rate_fps(60.0)),
                    LoopMode::Rate { .. } => app.set_loop_mode(LoopMode::wait(3)),
                }
                println!("Loop mode switched to: {:?}", app.loop_mode());
                let title = format!("`LoopMode` Demonstration - `{:?}`", app.loop_mode());
                app.main_window().set_title(&title);
            }
            _ => (),
        },
        Event::Update(update) => {
            println!("{:?}", update);
        }
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    frame.clear(DARK_CHARCOAL);
    frame
}
