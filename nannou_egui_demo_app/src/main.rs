use nannou::prelude::*;
use nannou_egui::Egui;
use std::sync::{Arc, Mutex};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    egui: Egui,
    egui_demo_app: egui_demo_lib::DemoWindows,
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
    let egui = Egui::from_window(&window);
    let egui_demo_app = egui_demo_lib::DemoWindows::default();

    let proxy = Arc::new(Mutex::new(app.create_proxy()));
    egui.ctx().set_request_repaint_callback(move |_| {
        proxy.lock().unwrap().wakeup().unwrap();
    });
    Model {
        egui,
        egui_demo_app,
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let Model {
        ref mut egui,
        ref mut egui_demo_app,
        ..
    } = *model;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui_demo_app.ui(&ctx.context());
    let _ = ctx.end();
}

fn view(_app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    model.egui.draw_to_frame(&frame).unwrap();
}
