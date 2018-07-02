extern crate nannou;

use nannou::prelude::*;

// This is how you make a global constant value. Accessible anywhere in your app
const GLOBAL: i32 = 10;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    foo: i32,
    bar: f64,
}

fn model(_app: &App) -> Model {
    // Initialise our models variables
    let foo = 80;
    let bar = 3.14;

    // Construct and return the model with our initialised values
    Model { foo, bar }
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => {
            match event {
                // KEY EVENTS
                KeyPressed(_key) => {
                    println!("foo = {}", model.foo);
                    println!("bar = {}", model.bar);
                }

                KeyReleased(_key) => {
                    let local_var = 94;
                    println!("local_variable to KeyReleased = {}", local_var);
                }

                // MOUSE EVENTS
                MousePressed(_button) => {
                    println!("global scope: GLOBAL = {}", GLOBAL);
                }

                _other => (),
            }
        }

        // update gets called just before view every frame
        Event::Update(_dt) => {}

        _ => (),
    }
    model
}

fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    // Color the window gray.
    frame.clear_all(DARK_CHARCOAL);
    // Return the drawn frame.
    frame
}
