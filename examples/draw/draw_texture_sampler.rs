//! A simple as possible example demonstrating how to use the `draw` API to display a texture.

use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    app.new_window().size(512, 512).view(view).build().unwrap();
    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path().unwrap();
    let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    let texture = wgpu::Texture::from_path(app, img_path).unwrap();
    Model { texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    draw.background().color(BLACK);
    let win = app.main_window();
    let win_r = win.rect();

    // Let's choose the address mode based on the mouse position.
    let address_mode = match map_range(app.mouse().y, win_r.top(), win_r.bottom(), 0.0, 3.0) as i8 {
        0 => wgpu::AddressMode::ClampToEdge,
        1 => wgpu::AddressMode::Repeat,
        _ => wgpu::AddressMode::MirrorRepeat,
    };

    // Create a sampler with the chosen address mode.
    let sampler = wgpu::SamplerBuilder::new()
        .address_mode(address_mode)
        .into_descriptor();

    // At any point during drawing, we can create a new `draw` context that will let us use a
    // different sampler.
    let draw = app.draw();
    let draw = draw.sampler(sampler);

    // Change the texture coordinates to sample outside the texture. This will demonstrate how the
    // sampler behaves when sampling beyond the bounds of the texture in each of the different
    // address modes. By default, the bounds of the texture coordinates are 0.0 to 1.0. We will
    // triple the size.
    let area = geom::Rect::from_x_y_w_h(0.5, 0.5, app.time.sin() * 10.0, app.time.sin() * 10.0);

    draw.texture(&model.texture).area(area);

    // Draw the current address mode in the bottom left corner.
    let text = format!("Address mode: {:?}", address_mode);
    draw.text(&text)
        .wh(win_r.wh() * 0.95)
        .left_justify()
        .align_text_bottom();


}
