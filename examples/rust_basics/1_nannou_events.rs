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
    app.new_window().event(event).view(view).build().unwrap();
    Model
}

// Update the state of your application here. By default, this gets called right before `view`.
fn update(_app: &App, _model: &mut Model, _update: Update) {}

// We can also update the application based on events received by the window like key presses and
// mouse movement here.
fn event(_app: &App, _model: &mut Model, event: WindowEvent) {
    // Print events as they occur to the console
    println!("{:?}", event);

    // We can `match` on the event to do something different depending on the kind of event.
    match event {
        // Keyboard events
        KeyPressed(_key) => {}
        KeyReleased(_key) => {}
        ReceivedCharacter(_char) => {}

        // Mouse events
        MouseMoved(_pos) => {}
        MousePressed(_button) => {}
        MouseReleased(_button) => {}
        MouseWheel(_amount, _phase) => {}
        MouseEntered => {}
        MouseExited => {}

        // Touch events
        Touch(_touch) => {}
        TouchPressure(_pressure) => {}

        // Window events
        Moved(_pos) => {}
        Resized(_size) => {}
        HoveredFile(_path) => {}
        DroppedFile(_path) => {}
        HoveredFileCancelled => {}
        Focused => {}
        Unfocused => {}
        Closed => {}
        Occluded(_val) => {}
    }
}

// Put your drawing code, called once per frame, per window.
fn view(_app: &App, _model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
