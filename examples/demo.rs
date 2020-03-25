use nannou::prelude::*;
use nannou_isf::IsfPipeline;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    watch: hotglsl::Watch,
    isf_pipeline: IsfPipeline,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    // Watch the shader directory.
    let assets = app.assets_path().unwrap();
    let shader_dir = assets.join("glsl");
    let watch = hotglsl::watch(&shader_dir).unwrap();

    // Create the hotloaded render pipeline.
    let window = app.main_window();
    let device = window.swap_chain_device();
    let vs_path = shader_dir.join("shader.vert");
    let fs_path = shader_dir.join("shader.frag");
    let dst_format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();
    let isf_pipeline = IsfPipeline::new(
        device,
        vs_path,
        fs_path,
        dst_format,
        sample_count,
    );

    Model {
        watch,
        isf_pipeline,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.swap_chain_device();
    // Feed new compilation result to the render pipeline for hotloading.
    let compilation_results = model.watch.compile_touched().unwrap();
    model.isf_pipeline.update_shaders(device, compilation_results);
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    // Encode a command to draw the render pipeline to the frame's texture.
    model.isf_pipeline.encode_to_frame(&frame);

    // Draw some feedback.
    let win = app.window_rect();
    let draw = app.draw();
    let (vs_string, vs_color) = match model.isf_pipeline.vs_compile_err() {
        None => (format!("Vertex Shader: Compiled Successfully!"), GREEN),
        Some(e) => (format!("Vertex Shader:\n{}", e), RED),
    };
    let (fs_string, fs_color) = match model.isf_pipeline.fs_compile_err() {
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
