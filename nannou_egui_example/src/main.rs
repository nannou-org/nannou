use nannou::prelude::*;
use nannou_egui::{self, egui};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

pub fn main() {
    nannou::app(model)
        .update(update)
        .size(WIDTH as u32, HEIGHT as u32)
        .run();
}

struct Model {
    egui_backend: nannou_egui::EguiBackend,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let window_id = app
        .new_window()
        .title("Nannou + Egui")
        .msaa_samples(1)
        .raw_event(raw_window_event) // This is where we forward all raw events for egui to process them
        .view(view) // The function that will be called for presenting graphics to a frame.
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();

    Model {
        egui_backend: nannou_egui::EguiBackend::new(
            window.swap_chain_device(),
            window.inner_size_pixels().0,
            window.inner_size_pixels().1,
            window.scale_factor() as f64,
        ),
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    model
        .egui_backend
        .update_time(update.since_last.as_secs_f64());

    let ctx = model.egui_backend.context();
    egui::Window::new("EGUI + Nannou window")
        .resizable(false)
        .collapsible(false)
        .fixed_size(egui::vec2(630.0, 800.0))
        .default_pos(egui::pos2(0.0, 0.0))
        .show(&ctx, |ui| {
            ui.label("Hello world It works :D!");
        });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui_backend.handle_event(event);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // frame.clear(BLACK);

    draw.ellipse().x_y(100.0, 100.0).color(WHITE);

    draw.to_frame(app, &frame).unwrap();

    model.egui_backend.draw_ui_to_frame(&frame);
    frame.submit();
}
