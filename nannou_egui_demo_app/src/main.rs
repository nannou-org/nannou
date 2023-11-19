use nannou::prelude::*;
use nannou_egui::{egui_wgpu_backend::epi::App as EguiApp, Egui};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    egui: Egui,
    egui_demo_app: egui_demo_lib::WrapApp,
}

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::wait());
    let w_id = app
        .new_window()
        .raw_event(raw_window_event)
        .view(view)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();
    let mut egui = Egui::from_window(&window);
    let mut egui_demo_app = egui_demo_lib::WrapApp::default();
    let proxy = app.create_proxy();
    egui.do_frame_with_epi_frame(proxy, |ctx, epi_frame| {
        egui_demo_app.setup(&ctx, epi_frame, None);
    });
    Model {
        egui,
        egui_demo_app,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(app: &App, model: &mut Model, update: Update) {
    let Model {
        egui,
        egui_demo_app,
        ..
    } = model;
    egui.set_elapsed_time(update.since_start);
    let proxy = app.create_proxy();
    egui.do_frame_with_epi_frame(proxy, |ctx, frame| {
        egui_demo_app.update(&ctx, frame);
    });
}

fn view(_app: &App, model: &Model, frame: Frame) {
    model.egui.draw_to_frame(&frame).unwrap();
}
