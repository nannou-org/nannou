extern crate nannou;

use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou::rand::random;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    ui: nannou::Ui,
    ids: Ids,
    bg_color: nannou::ui::Color,
}

struct Ids {
    button: widget::Id,
    text: widget::Id,
    background: widget::Id,
}

fn model(app: &App) -> Model {
    // Set the loop mode to wait for events, an energy-efficient option for pure-GUI apps.
    app.set_loop_mode(LoopMode::wait(3));

    // Create the UI.
    let mut ui = app.new_ui().build().unwrap();

    // Generate some ids for our widgets.
    let ids = Ids {
        button: ui.generate_widget_id(),
        text: ui.generate_widget_id(),
        background: ui.generate_widget_id(),
    };

    // Init background color
    let bg_color = color::rgba(1.0,0.0,1.0,1.0);

    Model { ui, ids, bg_color }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
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
            widget::Canvas::new().color(model.bg_color).set(model.ids.background, ui);

            // Draw the button and increment `count` if pressed.
            for _click in widget::Button::new()
                .up_from(model.ids.text,100.0)
                .w_h(180.0, 80.0)
                .label("random bg color")
                .set(model.ids.button, ui)
            {
                model.bg_color = color::rgba(random(),random(),random(),1.0);
            }

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
fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Draw the state of the `Ui` to the frame.
    model.ui.draw_to_frame(app, &frame).unwrap();
    // Return the drawn frame.
    frame
}
