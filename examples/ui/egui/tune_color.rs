use nannou::prelude::*;
use nannou_egui::{egui, Egui};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    window: Entity,
    radius: f32,
    color: Hsva,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let window = app
        .new_window()
        .title("Nannou + Egui")
        .size(WIDTH as u32, HEIGHT as u32)
        .view(view) // The function that will be called for presenting graphics to a frame.
        .build();

    Model {
        window,
        radius: 40.0,
        color: Color::hsv(10.0, 0.5, 1.0),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let Model {
        window,
        ref mut radius,
        ref mut color,
    } = *model;

    let mut egui_ctx = app.egui_for_window(window);
    let ctx = &egui_ctx.get_mut();
    egui::Window::new("EGUI window")
        .default_size(egui::vec2(0.0, 200.0))
        .show(&ctx, |ui| {
            ui.separator();
            ui.label("Tune parameters with ease");
            ui.add(egui::Slider::new(radius, 10.0..=100.0).text("Radius"));
            edit_hsv(ui, color);
        });
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();

    draw.background().color(BLACK);

    draw.ellipse()
        .x_y(100.0, 100.0)
        .radius(model.radius)
        .color(model.color);
}

fn edit_hsv(ui: &mut egui::Ui, color: &mut Hsva) {
    let mut egui_hsv = egui::ecolor::Hsva::new(
        color.hue.to_positive_radians() as f32 / (std::f32::consts::PI * 2.0),
        color.saturation,
        color.value,
        1.0,
    );

    if egui::color_picker::color_edit_button_hsva(
        ui,
        &mut egui_hsv,
        egui::color_picker::Alpha::Opaque,
    )
    .changed()
    {
        *color = Color::hsv(egui_hsv.h, egui_hsv.s, egui_hsv.v).into();
    }
}
