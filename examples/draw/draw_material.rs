use nannou::prelude::*;
use nannou::prelude::light_consts::lux::DIRECT_SUNLIGHT;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    window: Entity,
    light: Entity,
}

fn model(app: &App) -> Model {
    let light = app.new_light().color(WHITE).illuminance(1000.0).build();
    let window = app
        .new_window()
        .size_pixels(1024, 512)
        .light(light)
        .view(view)
        .build();

    Model { light, window }
}

fn update(app: &App, model: &mut Model) {
    let light = app.light(model.light);
    let window_rect = app.window_rect();
    let window_center = window_rect.xy();
    let mouse_x = app.mouse().x;
    let mouse_y = app.mouse().y;
    let norm_mouse_x = (mouse_x / window_rect.w()) + 0.5;
    let norm_mouse_y = (mouse_y / window_rect.h()) + 0.5;

    // Calculate the light's position based on time t
    let time = app.elapsed_seconds();
    let radius = window_rect.w().min(window_rect.h()) * 0.4;
    let light_x = window_center.x + radius * (time * 2.0 * PI).cos();
    let light_y = window_center.y + radius * (time * 2.0 * PI).sin();

    let light_color = Color::hsl((1.0 - norm_mouse_x) * 360.0, 1.0, 0.5);
    light
        .color(light_color)
        .illuminance(norm_mouse_y * DIRECT_SUNLIGHT)
        .x_y_z(light_x, light_y, 1000.0)
        .look_at(Vec2::ZERO);
}

fn view(app: &App, model: &Model) {
    // Begin drawing
    let draw = app.draw();

    for y in -2..=2 {
        for x in -5..=5 {
            let x01 = (x + 5) as f32 / 10.0;
            let y01 = (y + 2) as f32 / 2.0;

            draw.ellipse()
                .w_h(50.0, 50.0)
                .x_y_z(x as f32 * 100.0, y as f32 * 100.0, 10.0)
                .roughness(x01)
                .metallic(y01)
                .color(Color::gray(0.5));
        }
    }
}
