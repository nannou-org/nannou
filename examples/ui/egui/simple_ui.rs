use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Settings {
    resolution: u32,
    scale: f32,
    rotation: f32,
    color: Srgba,
    position: Vec2,
}

struct Model {
    settings: Settings,
    window: Entity,
}

fn model(app: &App) -> Model {
    // Create window
    let window = app.new_window().primary().view(view).build();

    Model {
        window,
        settings: Settings {
            resolution: 10,
            scale: 200.0,
            rotation: 0.0,
            color: WHITE,
            position: vec2(0.0, 0.0),
        },
    }
}

fn update(app: &App, model: &mut Model) {
    let settings = &mut model.settings;

    let mut egui_ctx = app.egui_for_window(model.window);
    let ctx = egui_ctx.get_mut();

    egui::Window::new("Settings").show(&ctx, |ui| {
        // Resolution slider
        ui.label("Resolution:");
        ui.add(egui::Slider::new(&mut settings.resolution, 1..=40));

        // Scale slider
        ui.label("Scale:");
        ui.add(egui::Slider::new(&mut settings.scale, 0.0..=1000.0));

        // Rotation slider
        ui.label("Rotation:");
        ui.add(egui::Slider::new(&mut settings.rotation, 0.0..=360.0));

        // Random color button
        let clicked = ui.button("Random color").clicked();

        if clicked {
            settings.color = Color::srgb(random(), random(), random()).into();
        }
    });
}

fn view(app: &App, model: &Model) {
    let settings = &model.settings;

    let draw = app.draw();
    draw.background().color(BLACK);

    let rotation_radians = deg_to_rad(settings.rotation);
    draw.ellipse()
        .resolution(settings.resolution as f32)
        .xy(settings.position)
        .color(settings.color)
        .rotate(-rotation_radians)
        .radius(settings.scale);
}
