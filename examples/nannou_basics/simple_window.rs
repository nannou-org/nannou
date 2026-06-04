use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: Entity,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let _window = app
        .new_window()
        .size(512, 512)
        .title("nannou")
        .view(view) // The function that will be called for presenting graphics to a frame.
        .focused(focus) // The function that will be called when the window receives events.
        .build();
    Model { _window }
}

// Handle events related to the window and update the model if necessary
fn focus(_app: &App, _model: &mut Model) {
    println!("Focused");
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, _model: &Model) {
    let draw = _app.draw();
    draw.background().color(CORNFLOWER_BLUE);
}
