//! A simple example demonstrating the behaviour of the three `LoopMode` variants supported by
//! nannou.
//!
//! The `LoopMode` determines how the nannou application loop is driven.
//!
//! See the `LoopMode` docs for more details:
//!
//! https://docs.rs/nannou/latest/nannou/app/enum.LoopMode.html

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model;

fn model(app: &App) -> Model {
    // Start in `Wait` mode. In other words, don't keep looping, just wait for events.
    app.set_loop_mode(LoopMode::wait(3));
    let _window = app
        .new_window()
        .with_title(format!(
            "`LoopMode` Demonstration - `{:?}`",
            app.loop_mode()
        ))
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();
    Model
}

fn update(_app: &App, _model: &mut Model, update: Update) {
    println!("{:?}", update);
}

fn key_pressed(app: &App, _model: &mut Model, _key: Key) {
    // Switch to the next loop mode on key pressed.
    match app.loop_mode() {
        LoopMode::Wait { .. } => app.set_loop_mode(LoopMode::refresh_sync()),
        LoopMode::RefreshSync { .. } => app.set_loop_mode(LoopMode::rate_fps(60.0)),
        LoopMode::Rate { .. } => app.set_loop_mode(LoopMode::loop_once()),
        LoopMode::NTimes { .. } => app.set_loop_mode(LoopMode::wait(3)),
    }
    println!("Loop mode switched to: {:?}", app.loop_mode());
    let title = format!("`LoopMode` Demonstration - `{:?}`", app.loop_mode());
    app.main_window().set_title(&title);
}

fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    frame.clear(DARK_CHARCOAL);
    frame
}
