extern crate nannou;

use nannou::prelude::*;

// every rust program has to have a main function which gets
// called when the program is run.
fn main() {
    nannou::run(model, event, view);
}

// model represents the state of our app
struct Model;

// put your setup code here, to run once before the application loop:
fn model(_app: &App) -> Model {
    Model
}

// put your update code here, to set variables and handle
// keyboard and mouse events before drawing each frame:
fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            // Print events as they occur to the console
            println!("{:#?}", event);

            match event {
                // KEY EVENTS
                KeyPressed(_key) => {}

                KeyReleased(_key) => {}

                // MOUSE EVENTS
                MouseMoved(_pos) => {}

                MouseDragged(_pos, _button) => {}

                MousePressed(_button) => {}

                MouseReleased(_button) => {}

                MouseEntered => {}

                MouseExited => {}

                // WINDOW EVENTS
                Resized(_size) => {}

                Moved(_pos) => {}

                _other => (),
            }
        }

        // update gets called just before view every frame
        Event::Update(_dt) => {}

        _ => (),
    }
    model
}

// put your main code here, to run repeatedly:
fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    // Clear the window to a dark charcoal color.
    frame.clear_all(DARK_CHARCOAL);
    // Return the drawn frame.
    frame
}
