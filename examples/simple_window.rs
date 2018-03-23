extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::app(model, event, view).run();
}

struct Model {
    window: WindowId,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let window = app.new_window().with_title("nannou").build().unwrap();
    Model { window }
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        // Handle window events like mouse, keyboard, resize, etc here.
        Event::WindowEvent { simple: Some(event), .. } => {
            println!("{:?}", event);
        },
        // `Update` the model here.
        Event::Update(_update) => {
        },
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(_app: &App, model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it dark charcoal.
    frame.window(model.window).unwrap().clear(DARK_CHARCOAL);
    // Return the cleared frame.
    frame
}
