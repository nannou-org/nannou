//! A simple as possible example demonstrating how to use the `draw` API to display a texture.

use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    texture: Handle<Image>,
}

fn model(app: &App) -> Model {
    // Create a new window!
    app.new_window().size(512, 512).view(view).build();
    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path();
    let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    let texture = app.assets().load(img_path);
    Model { texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let win = app.window_rect();
    draw.rect().x_y(win.x(), win.y()).texture(&model.texture);
}
