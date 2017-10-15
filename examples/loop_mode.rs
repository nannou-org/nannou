extern crate nannou;

use nannou::{App, Event, Frame, LoopMode, WindowEvent, WindowId, VirtualKeyCode as Key, ElementState};

fn main() {
    nannou::run(model, update, draw);
}

struct Model {
    window: nannou::WindowId,
}

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::wait(3));
    let window = app.new_window().build().unwrap();
    Model { window }
}

fn update(app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent(id, event) => {
            println!("{:?}", event);
            match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if let (ElementState::Pressed, Some(_)) = (input.state, input.virtual_keycode) {
                        match app.loop_mode() {
                            LoopMode::Rate { .. } => app.set_loop_mode(LoopMode::wait(3)),
                            LoopMode::Wait { .. } => app.set_loop_mode(LoopMode::rate_fps(60.0)),
                        }
                    }
                },
                _ => (),
            }
        },
        Event::Update(update) => {
            println!("{:?}", &update);
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
