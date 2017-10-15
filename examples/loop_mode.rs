extern crate nannou;

use nannou::{App, Event, Frame, LoopMode, WindowEvent, WindowId, VirtualKeyCode, ElementState};

fn main() {
    nannou::run::<Model, Event>(model, update, draw);
}

struct Model {
    window: nannou::WindowId,
}

fn model(app: &App) -> Model {
    let window = app.new_window().build().unwrap();
    Model { window }
}

fn update(app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent(id, event) => {
            println!("{:?}", event);
            window_event(app, model, id, event)
        },
        Event::Update(update) => {
            println!("{:?}", &update);
            model
        },
        _ => model,
    }
}

fn window_event(app: &App, model: Model, _id: WindowId, event: WindowEvent) -> Model {
    match event {
        WindowEvent::KeyboardInput { input, .. } => match input.state {
            ElementState::Pressed => match input.virtual_keycode {
                Some(VirtualKeyCode::Space) => {
                    match app.loop_mode() {
                        LoopMode::Rate { .. } => app.set_loop_mode(LoopMode::wait(3)),
                        LoopMode::Wait { .. } => app.set_loop_mode(LoopMode::rate_fps(60.0)),
                    }
                    println!("Switched LoopMode to: {:?}", app.loop_mode());
                },
                _ => (),
            },
            ElementState::Released => {},
        },
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn draw(_app: &App, model: &Model, frame: Frame) -> Frame {
    {
        // Our app only has one window, so retrieve this part of the `Frame`.
        let mut window_frame = frame.window(model.window).unwrap();
        // Paint it! (red, green, blue, alpha).
        window_frame.clear_color(1.0, 0.0, 0.0, 1.0);
    }
    // Return the drawn frame.
    frame
}
