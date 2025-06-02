//! A simple as possible example demonstrating how to use the `draw` API to display a texture.

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    texture: Handle<Image>,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    app.new_window().size(512, 512).view(view).primary().build();
    // Load the image from disk and upload it to a GPU texture.
    let texture = app.asset_server().load("images/nature/nature_1.jpg");
    Model { texture }
}

fn update(app: &App, model: &mut Model) {
    let mut images = app.assets_mut::<Image>();
    let Some(image) = images.get_mut(&model.texture) else {
        return;
    };

    let win = app.main_window();
    let win_r = win.rect();

    // Let's choose the address mode based on the mouse position.
    let address_mode = match map_range(app.mouse().y, win_r.top(), win_r.bottom(), 0.0, 3.0) as i8 {
        0 => ImageAddressMode::ClampToEdge,
        1 => ImageAddressMode::Repeat,
        _ => ImageAddressMode::MirrorRepeat,
    };

    let descriptor = ImageSamplerDescriptor {
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        ..default()
    };

    image.sampler = ImageSampler::Descriptor(descriptor);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);

    // Change the texture coordinates to sample outside the texture. This will demonstrate how the
    // sampler behaves when sampling beyond the bounds of the texture in each of the different
    // address modes. By default, the bounds of the texture coordinates are 0.0 to 1.0. We will
    // triple the size.
    let area = geom::Rect::from_x_y_w_h(0.5, 0.5, app.time().sin() * 10.0, app.time().sin() * 10.0);
    let window_rect = app.main_window().rect();

    draw.rect()
        .w_h(window_rect.w(), window_rect.h())
        .texture(&model.texture)
        .area(area);
}
