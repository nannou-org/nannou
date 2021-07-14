use nannou::prelude::*;
use nannou_egui::egui_wgpu_backend::epi::App as EguiApp;
use nannou_egui::EguiBackend;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    egui_backend: EguiBackend,
    egui_app: egui_demo_lib::WrapApp,
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
    let proxy = app.create_proxy();
    let mut egui_backend = EguiBackend::from_window(&window, proxy);
    let mut egui_app = egui_demo_lib::WrapApp::default();
    egui_backend.with_ctxt_and_frame(|ctx, frame| {
        egui_app.setup(ctx, frame, None);
    });
    Model {
        egui_backend,
        egui_app,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui_backend.handle_event(event);
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let Model {
        ref mut egui_backend,
        ref mut egui_app,
        ..
    } = *model;

    egui_backend.begin_frame();
    egui_backend.with_ctxt_and_frame(|ctx, frame| {
        egui_app.update(ctx, frame);
    });
    egui_backend.end_frame();
}

fn view(_app: &App, model: &Model, frame: Frame) {
    model.egui_backend.draw_ui_to_frame(&frame);
}
