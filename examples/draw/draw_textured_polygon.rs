use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    window_id: Entity,
    texture: Handle<Image>,
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().size(512, 512).view(view).build();

    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path();
    let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    let texture = app.assets_mut().load(img_path);

    Model { window_id, texture }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let win_rect = app.window_rect();
    let draw = app.draw();
    draw.background().color(DIM_GRAY);

    // We'll make a wave from an ellipse with a wave added onto its circumference.
    let resolution = win_rect.right().floor();
    let rect = geom::Rect::from_wh(vec2(1.0, 1.0));
    let ellipse = geom::Ellipse::new(rect, resolution).circumference();

    // The wave's frequency and amplitude are derived from the mouse position.
    let freq = map_range(app.mouse().x, win_rect.left(), win_rect.right(), 1.0, 20.0);
    let amp = map_range(app.mouse().x, win_rect.bottom(), win_rect.top(), 0.0, 0.5);
    let wave = (0..resolution as usize).map(|i| {
        let phase = i as f32 / resolution;
        (phase * freq * PI * 2.0).sin() * amp
    });

    // Combine the ellipse with the wave.
    let points = ellipse.zip(wave).map(|(point, wave)| {
        // Base the tex coords on the non-wavey points.
        // This will make the texture look wavey.
        let point = Point2::from(point);
        let tex_coords = [point.x + 0.5, 1.0 - (point.y + 0.5)];
        // Apply the wave to the points.
        let point = point + point * wave;
        (point, Color::WHITE, tex_coords)
    });

    // Scale the points up to half the window size.
    let ellipse_side = win_rect.w().min(win_rect.h()) * 0.75;
    draw.scale(ellipse_side)
        .polygon()
        .points_vertex(points)
        .texture(&model.texture)
        .rotate(app.elapsed_seconds() * 0.25);
}
