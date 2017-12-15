extern crate nannou;

use nannou::prelude::*;

// This is how you make a global constant value. Accessible anywhere in your app
const GLOBAL: i32 = 10;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    // Declare the variables that our model contains here
    window: WindowId,
    foo: i32,
    bar: f64,
}

fn model(app: &App) -> Model {
    // Initialise our models variables 
    let foo = 80;
    let bar = 3.14;

    let window = app.new_window().with_dimensions(640,480).build().unwrap();

    // Construct and return the model with our initialised values
    Model { window, foo, bar }
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent { simple: Some(event), .. } =>{

            match event {

                /*            KEY EVENTS              */
                KeyPressed(_key) => {
                    println!("foo = {}", model.foo);
                    println!("bar = {}", model.bar);
                },

                KeyReleased(_key) => {
                    let local_var = 94;
                    println!("local_variable to KeyReleased = {}", local_var);
                },

                /*            MOUSE EVENTS             */
                MousePressed(_button) => {
                    println!("global scope: GLOBAL = {}", GLOBAL);
                },
                
                _other => (),
            }
        },

        // update gets called just before view every frame
        Event::Update(_dt) => {
        },

        _ => (),
    } 
    model
}

fn view(_app: &App, model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it gray.
    frame.window(model.window).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);
    // Return the drawn frame.
    frame
}
