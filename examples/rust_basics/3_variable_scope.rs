use nannou::prelude::*;

// This is how you make a global constant value.
// Constant values are accessible anywhere in your app but cannot be changed.
const GLOBAL: i32 = 10;

fn main() {
    nannou::app(model).run();
}

struct Model {
    foo: i32,
    bar: f64,
}

fn model(app: &App) -> Model {
    // Make a window.
    app.new_window()
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_pressed(mouse_pressed)
        .view(view)
        .build()
        .unwrap();
    // Initialise our model's fields.
    let foo = 80;
    let bar = 3.14;
    // Construct and return the model with our initialised values.
    Model { foo, bar }
}

fn key_pressed(_app: &App, model: &mut Model, _key: KeyCode) {
    println!("foo = {}", model.foo);
    println!("bar = {}", model.bar);
}

fn key_released(_app: &App, _model: &mut Model, _key: KeyCode) {
    let local_var = 94;
    println!("local_variable to KeyReleased = {}", local_var);
}

fn mouse_pressed(_app: &App, _model: &mut Model, _button: MouseButton) {
    println!("global scope: GLOBAL = {}", GLOBAL);
}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(DIM_GRAY);
}
