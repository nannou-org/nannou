use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    _window_id: Entity,
    texture: Handle<Image>,
}

fn model(app: &App) -> Model {
    let _window_id = app.new_window().size(512, 512).view(view).build();

    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path();
    let img_path = assets.join("images").join("nature").join("nature_1.jpg");
    let texture = app.asset_server().load(img_path);

    Model {
        _window_id,
        texture,
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let win_rect = app.window_rect();
    let draw = app.draw();
    draw.background().color(DIM_GRAY);

    // Generate a spiral for the path to follow.
    // Modulate the frequency of the spiral with a wave over time.
    let wave = (app.time() * 0.125).cos();
    let freq = map_range(wave, -1.0, 1.0, 2.0, 20.0);
    let spiral_side = win_rect.w().min(win_rect.h()) * 0.5;
    let points = (0..spiral_side as u32).map(|i| {
        let phase = i as f32 / spiral_side;
        let mag = phase;
        let x = (phase * freq * PI * 2.0).sin();
        let y = (phase * freq * PI * 2.0).cos();
        let point = pt2(x, y) * mag;
        // Retrieve the texture points based on the position of the spiral.
        let tex_coords = [x * 0.5 + 0.5, 1.0 - (point.y * 0.5 + 0.5)];
        (point, Color::WHITE, tex_coords)
    });

    // Scale the points up to half the window size.
    draw.scale(spiral_side)
        .path()
        .stroke()
        .weight(0.9 / freq)
        .points_vertex(points)
        .texture(&model.texture)
        .rotate(app.time() * 0.25);

    // Draw to the frame!
}
