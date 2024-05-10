use nannou::prelude::*;

// Every rust program has to have a main function which gets called when the program is run.
// In the main function, we build the nannou app and run it.
fn main() {
    nannou::app(model).update(update).run();
}

// Model represents the state of our application. We don't have any state in this demonstration, so
// for now it is just an empty struct.
struct Model;

// This function is where we setup the application and create the `Model` for the first time.
fn model(app: &App) -> Model {
    // Create a window that can receive user input like mouse and keyboard events.
    app.new_window().view(view).build().unwrap();
    Model
}

// Update the state of your application here. By default, this gets called right before `view`.
fn update(_app: &App, _model: &mut Model) {}

// Put your drawing code, called once per frame, per window.
fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(DIM_GRAY);
}
