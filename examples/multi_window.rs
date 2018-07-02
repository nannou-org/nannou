extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, update, draw).run();
}

struct Model {
    a: WindowId,
    b: WindowId,
    c: WindowId,
}

fn model(app: &App) -> Model {
    let a = app.new_window().with_title("window a").build().unwrap();
    let b = app.new_window().with_title("window b").build().unwrap();
    let c = app.new_window().with_title("window c").build().unwrap();
    Model { a, b, c }
}

fn update(_app: &App, model: Model, event: Event) -> Model {
    match event {
        // Handle window events like mouse, keyboard, resize, etc here.
        Event::WindowEvent {
            id,
            simple: Some(event),
            ..
        } => {
            println!("Window {:?}: {:?}", id, event);
        }
        // `Update` the model here.
        Event::Update(_update) => {}
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn draw(_app: &App, model: &Model, frame: Frame) -> Frame {
    // Clear each window with a different colour.
    for (id, mut window_frame) in frame.windows() {
        match id {
            id if id == model.a => window_frame.clear(RED),
            id if id == model.b => window_frame.clear(GREEN),
            id if id == model.c => window_frame.clear(BLUE),
            _ => (),
        }
    }
    frame
}
