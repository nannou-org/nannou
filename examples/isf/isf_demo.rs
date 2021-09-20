// NOTE: This example is a work-in-progress

use nannou::prelude::*;
use nannou_isf::{IsfPipeline, IsfTime};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    watch: hotglsl::Watch,
    isf_pipeline: IsfPipeline,
    isf_time: IsfTime,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    // Watch the shader directory.
    let assets = app.assets_path().unwrap();
    let images_dir = assets.join("images");
    let shader_dir = assets.join("isf");
    let watch = hotglsl::watch(&shader_dir).unwrap();

    // Create the hotloaded render pipeline.
    let window = app.main_window();
    let device = window.device();
    let vs_path = None;
    let fs_path = shader_dir.join("Test-Float.fs");
    let dst_format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();
    let desc = wgpu::CommandEncoderDescriptor {
        label: Some("nannou_isf_pipeline_new"),
    };
    let mut encoder = device.create_command_encoder(&desc);
    let (dst_w, dst_h) = window.inner_size_pixels();
    let isf_pipeline = IsfPipeline::new(
        device,
        &mut encoder,
        vs_path,
        fs_path,
        dst_format,
        [dst_w, dst_h],
        sample_count,
        &images_dir,
    );
    window.queue().submit(Some(encoder.finish()));

    let isf_time = Default::default();

    Model {
        watch,
        isf_pipeline,
        isf_time,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let window = app.main_window();
    let device = window.device();
    // Feed new compilation result to the render pipeline for hotloading.
    let touched_shaders = model.watch.paths_touched().unwrap();
    let assets = app.assets_path().unwrap();
    let images_dir = assets.join("images");
    let desc = wgpu::CommandEncoderDescriptor {
        label: Some("nannou_isf_pipeline_update"),
    };
    let mut encoder = device.create_command_encoder(&desc);
    model
        .isf_pipeline
        .encode_update(device, &mut encoder, &images_dir, touched_shaders);
    window.queue().submit(Some(encoder.finish()));
    model.isf_time.time = update.since_start.secs() as _;
    model.isf_time.time_delta = update.since_last.secs() as _;
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    // Encode a command to draw the render pipeline to the frame's texture.
    model.isf_pipeline.encode_to_frame(&frame, model.isf_time);

    // Draw some feedback.
    let win = app.window_rect();
    let draw = app.draw();
    let (vs_string, vs_color) = match model.isf_pipeline.vs_err() {
        None => (format!("Vertex Shader: Compiled Successfully!"), GREEN),
        Some(e) => (format!("Vertex Shader:\n{}", e), RED),
    };
    let (fs_string, fs_color) = match model.isf_pipeline.fs_err() {
        None => (format!("Fragment Shader: Compiled Successfully!"), GREEN),
        Some(e) => (format!("Fragment Shader:\n{}", e), RED),
    };
    let win_p = win.pad(30.0);
    let text_area = geom::Rect::from_wh(win_p.wh()).top_left_of(win_p);
    draw.text(&vs_string)
        .xy(text_area.xy())
        .wh(text_area.wh())
        .font_size(16)
        .align_text_top()
        .left_justify()
        .color(vs_color);
    let text_area = text_area.shift_y(-30.0);
    draw.text(&fs_string)
        .xy(text_area.xy())
        .wh(text_area.wh())
        .font_size(16)
        .align_text_top()
        .left_justify()
        .color(fs_color);
    draw.to_frame(app, &frame).unwrap();
}
