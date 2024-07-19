use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    window: Entity,
    camera: Entity,
}

fn model(app: &App) -> Model {
    let camera = app
        .new_camera()
        // HDR is required for bloom to work.
        .hdr(true)
        // Pick a default bloom settings. This also can be configured manually.
        .bloom_settings(BloomSettings::OLD_SCHOOL)
        .build();

    let window = app
        .new_window()
        .camera(camera)
        .size_pixels(1024, 1024)
        .view(view)
        .build();

    Model { camera, window }
}

fn update(app: &App, model: &mut Model) {
    let camera = app.camera(model.camera);
    let window_rect = app.window_rect();
    let norm_mouse_y = (app.mouse().y / window_rect.w()) + 0.5;

    camera.bloom_intensity(norm_mouse_y.clamp(0.0, 0.8));
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();
    let window_rect = app.window_rect();
    let norm_mouse_x = (app.mouse().x / window_rect.w()) + 0.5;

    // Use the normalized mouse coordinate to create an initial color.
    let color_hsl = Color::hsl((1.0 - norm_mouse_x) * 360.0, 1.0, 0.5);

    // Convert the color to linear RGBA and multiply (for emissives, values outside of 1.0 are used).
    let mut color_linear_rgb: LinearRgba = color_hsl.into();
    color_linear_rgb = color_linear_rgb * 5.0;

    let t = app.time();

    draw.tri()
        .width(100.0)
        .emissive(color_linear_rgb)
        .color(WHITE);
}
