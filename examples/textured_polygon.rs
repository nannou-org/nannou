use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    window_id: window::Id,
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().size(512, 512).view(view).build().unwrap();

    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path().unwrap();
    let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    let texture = wgpu::Texture::from_path(app, img_path).unwrap();

    Model { window_id, texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
    let window = app.window(model.window_id).unwrap();
    let win_rect = window.rect();
    let draw = app.draw();

    // We'll make a wave from an ellipse with a wave added onto its circumference.
    let resolution = win_rect.right() as usize;
    let rect = geom::Rect::from_wh(vec2(1.0, 1.0));
    let ellipse = geom::Ellipse::new(rect, resolution).circumference();

    // The wave's frequency and amplitude are derived from the mouse position.
    let freq = map_range(app.mouse.x, win_rect.left(), win_rect.right(), 1.0, 20.0);
    let amp = map_range(app.mouse.y, win_rect.bottom(), win_rect.top(), 0.0, 0.5);
    let wave = (0..resolution).map(|i| {
        let phase = i as f32 / resolution as f32;
        (phase * freq * PI * 2.0).sin() * amp
    });

    // Combine the ellipse with the wave.
    let points = ellipse.zip(wave).map(|(point, wave)| {
        // Base the tex coords on the non-wavey points.
        // This will make the texture look wavey.
        let tex_coords = [point.x + 0.5, 1.0 - (point.y + 0.5)];
        // Apply the wave to the points.
        let point = point + point * wave;
        (point, tex_coords)
    });

    // Scale the points up to half the window size.
    let ellipse_side = win_rect.w().min(win_rect.h()) * 0.75;
    draw.scale(ellipse_side)
        .polygon()
        .points_textured(&model.texture, points)
        .rotate(app.time * 0.25);

    // Draw to the frame!
    draw.to_frame(app, &frame).unwrap();
}
