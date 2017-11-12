extern crate nannou;

use nannou::prelude::*;
use nannou::ui::prelude::*;

fn main() {
    nannou::run(model, update, draw);
}

struct Model {
    ui: nannou::Ui,
    ids: Ids,
}

struct Ids {
    text: widget::Id,
    background: widget::Id,
}

fn model(app: &App) -> Model {
    // Set the loop mode to wait for events, an energy-efficient option for pure-GUI apps.
    app.set_loop_mode(LoopMode::wait(3));

    // Create the window.
    let window = app.new_window().build().unwrap();

    // Create the UI.
    let mut ui = app.new_ui(window).build().unwrap();

    // Generate some ids for our widgets.
    let ids = Ids {
        text: ui.generate_widget_id(),
        background: ui.generate_widget_id(),
    };

    Model { ui, ids }
}

fn update(_app: &App, mut model: Model, event: Event) -> Model {
    match event {
        // Handle window events like mouse, keyboard, resize, etc here.
        Event::WindowEvent { simple: Some(event), .. } => {
            println!("{:?}", event);
        },
        // `Update` the model here.
        Event::Update(update) => {

            // Calling `set_widgets` allows us to instantiate some widgets.
            let ui = &mut model.ui.set_widgets();

            // Place a canvas as the background. A canvas auto-sizes to the window if no dimensions
            // were specified.
            widget::Canvas::new().color(color::DARK_BLUE).set(model.ids.background, ui);

            // Draw the update event using a `Text` widget.
            let text = format!("{:#?}", update);
            widget::Text::new(&text)
                .color(color::WHITE)
                .middle_of(model.ids.background)
                .set(model.ids.text, ui);
        },
        _ => (),
    }
    model
}

// Draw the state of your `Model` into the given `Frame` here.
fn draw(app: &App, model: &Model, frame: Frame) -> Frame {
    // Draw the state of the `Ui` to the frame.
    model.ui.draw_to_frame(app, &frame).unwrap();
    // Return the drawn frame.
    frame
}
