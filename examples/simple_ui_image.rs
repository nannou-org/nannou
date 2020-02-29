use nannou::prelude::*;
use nannou::ui::prelude::*;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    ui: Ui,
    widget_id: widget::Id,
    image_id: image::Id
}

fn model(app: &App) -> Model {
    // Create a new Ui and generate a widget id for the image.
    let mut ui = app.new_ui().build().unwrap();
    let widget_id = ui.generate_widget_id();

    // Generate the image path, and load the image using the image crate.
    let logo_path = app.assets_path().unwrap().join("images").join("Nannou.png");
    let image = nannou::image::open(logo_path).unwrap();

    // Upload it to the Ui.
    let image_id = ui.upload_image(app, &image).unwrap();

    Model {
        ui, image_id, widget_id
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let mut ui = model.ui.set_widgets();

    // Construct an Image primitive with the id from before and display it.
    widget::primitive::image::Image::new(model.image_id)
        .middle()
        .w_h(200.0, 200.0)
        .set(model.widget_id, &mut ui);
}

fn view(app: &App, model: &Model, frame: &Frame) {
    frame.clear(WHITE);
    model.ui.draw_to_frame(app, frame).unwrap();
}