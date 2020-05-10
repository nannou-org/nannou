// NOTE: This example is a work-in-progress

use fps_ticker::Fps;
use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou_isf::{IsfPipeline, IsfTime};
use std::path::{Path, PathBuf};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _ui_window_id: window::Id,
    shader_window_id: window::Id,
    watch: hotglsl::Watch,
    ui: Ui,
    ids: Ids,
    fps: Fps,
    isf_shader_paths: Vec<PathBuf>,
    isf: Option<IsfState>,
}

struct State<'a> {
    isf: &'a mut Option<IsfState>,
    isf_shader_paths: &'a [PathBuf],
    fps: &'a Fps,
    assets: &'a Path,
    shader_window: &'a Window,
}

struct IsfState {
    path: PathBuf,
    pipeline: IsfPipeline,
    time: IsfTime,
}

widget_ids! {
    struct Ids {
        background_canvas,
        isf_shader_text,
        isf_shader_select_list,
        isf_status_text,
        vs_status_text,
        fs_status_text,
        separator0,
        fps_avg_text,
        fps_min_text,
        fps_max_text,
        separator1,
        separator2,
    }
}

const WIN_H: u32 = 720;
const UI_WIN_W: u32 = 240;
const SHADER_WIN_W: u32 = 1280;

fn model(app: &App) -> Model {
    let ui_window_id = app
        .new_window()
        .title("nannou isf_demo.rs - GUI")
        .size(UI_WIN_W, WIN_H)
        .view(ui_view)
        .build()
        .unwrap();
    let shader_window_id = app
        .new_window()
        .title("nannou isf_demo.rs - Shader")
        .size(SHADER_WIN_W, WIN_H)
        .view(shader_view)
        .build()
        .unwrap();

    // Layout the windows side by side.
    let ui_window = app.window(ui_window_id).unwrap();
    let shader_window = app.window(shader_window_id).unwrap();
    let (ui_x, ui_y) = ui_window.outer_position_pixels().unwrap();
    shader_window.set_outer_position_pixels(ui_x + UI_WIN_W as i32 + 20, ui_y);

    // Find the images directory.
    let assets = app.assets_path().unwrap();
    let images_dir = images_dir(&assets);

    // Watch the directory specified by the user, or default to `assets/isf`.
    let shader_dir = match std::env::args().nth(1) {
        Some(user_path) => Path::new(&user_path).to_path_buf(),
        None => assets.join("isf"),
    };

    let isf_shader_paths = find_isf_shaders(&shader_dir);
    let watch = hotglsl::watch(&shader_dir).unwrap();

    // Initialise with the first shader in the list.
    let isf = isf_shader_paths
        .iter()
        .next()
        .map(|fs_path| create_isf_state(&shader_window, fs_path.clone(), &images_dir));

    // Create the `Ui` for controlling inputs.
    let mut ui = app.new_ui().window(ui_window_id).build().unwrap();
    let ids = Ids::new(ui.widget_id_generator());

    let fps = Fps::default();

    Model {
        _ui_window_id: ui_window_id,
        shader_window_id,
        watch,
        isf,
        ui,
        ids,
        fps,
        isf_shader_paths,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let assets = app.assets_path().unwrap();
    let shader_window = app.window(model.shader_window_id).unwrap();

    // Update the GUI.
    {
        let Model {
            ref mut ui,
            ref ids,
            ref mut isf,
            ref isf_shader_paths,
            ref fps,
            ..
        } = *model;
        let state = State {
            isf: isf,
            isf_shader_paths,
            fps,
            assets: &assets,
            shader_window: &*shader_window,
        };
        gui(&mut ui.set_widgets(), ids, state);
    }

    if let Some(ref mut isf) = model.isf {
        let shader_window = app.window(model.shader_window_id).unwrap();
        let device = shader_window.swap_chain_device();
        // Feed new compilation result to the render pipeline for hotloading.
        let touched_shaders = model.watch.paths_touched().unwrap();
        let images_dir = images_dir(&assets);
        let desc = wgpu::CommandEncoderDescriptor {
            label: Some("nannou_isf_pipeline_update"),
        };
        let mut encoder = device.create_command_encoder(&desc);
        isf.pipeline
            .encode_update(device, &mut encoder, &images_dir, touched_shaders);
        shader_window.swap_chain_queue().submit(&[encoder.finish()]);
        isf.time.time = update.since_start.secs() as _;
        isf.time.time_delta = update.since_last.secs() as _;
    }
}

fn images_dir(assets: &Path) -> PathBuf {
    assets.join("images")
}

fn ui_view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    model.ui.draw_to_frame(app, &frame).unwrap();
}

fn gui(ui: &mut UiCell, ids: &Ids, state: State) {
    const PAD: Scalar = 20.0;
    const STATUS_FONT_SIZE: u32 = 14;

    fn status_text(s: &str) -> widget::Text {
        widget::Text::new(s).font_size(STATUS_FONT_SIZE)
    }

    fn fps_to_rgb(fps: f64) -> (f32, f32, f32) {
        let r = clamp(map_range(fps, 0.0, 60.0, 1.0, 0.0), 0.0, 1.0);
        let g = clamp(map_range(fps, 0.0, 60.0, 0.0, 1.0), 0.0, 1.0);
        let b = 0.5;
        (r, g, b)
    }

    widget::Canvas::new()
        .border(0.0)
        .rgb(0.11, 0.12, 0.13)
        .set(ids.background_canvas, ui);

    let bg_id = ids.background_canvas;
    let bg_rect = ui.rect_of(ids.background_canvas).unwrap();
    let bg_rect_pad = bg_rect.pad(PAD);

    let separator = || -> widget::Line {
        let start = [bg_rect_pad.left(), 0.0];
        let end = [bg_rect_pad.right(), 0.0];
        widget::Line::centred(start, end)
            .down(PAD)
            .parent(bg_id)
            .color(ui::color::DARK_CHARCOAL)
            .align_middle_x_of(bg_id)
    };

    // ISF Shader Selection
    // ------------------------------------------------------------------------

    widget::Text::new("ISF Shader")
        .padded_w_of(ids.background_canvas, PAD)
        .font_size(14)
        .top_left_with_margin_on(ids.background_canvas, PAD)
        .color(ui::color::WHITE)
        .set(ids.isf_shader_text, ui);

    // Instantiate the `ListSelect` widget.
    let num_items = state.isf_shader_paths.len();
    let item_h = 30.0;
    let font_size = (item_h * 2.0 / 5.0) as ui::FontSize;
    let (mut events, scrollbar) = widget::ListSelect::single(num_items)
        .mid_left_with_margin_on(ids.background_canvas, PAD)
        .down(PAD * 0.8)
        .padded_w_of(ids.background_canvas, PAD)
        .flow_down()
        .h(item_h * 3.0)
        .item_size(item_h)
        .scrollbar_next_to()
        .set(ids.isf_shader_select_list, ui);

    // Handle the `ListSelect`s events.
    let mut selected_ix = state
        .isf
        .as_ref()
        .and_then(|isf| state.isf_shader_paths.iter().position(|p| p == &isf.path));
    while let Some(event) = events.next(ui, |i| Some(i) == selected_ix) {
        use nannou::ui::widget::list_select::Event;
        match event {
            Event::Item(item) => {
                let path = &state.isf_shader_paths[item.i];
                let label = path
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                    .unwrap_or("<invalid-file_name>");
                let color = match Some(item.i) == selected_ix {
                    true => ui::color::DARK_BLUE,
                    false => ui::color::rgb(0.06, 0.065, 0.07),
                };
                let button = widget::Button::new()
                    .border(0.0)
                    .color(color)
                    .label(label)
                    .label_x(position::Relative::Place(position::Place::Start(Some(
                        PAD * 0.5,
                    ))))
                    .label_font_size(font_size)
                    .label_color(ui::color::WHITE);
                item.set(button, ui);
            }
            Event::Selection(new_ix) => {
                let isf_path = state.isf_shader_paths[new_ix].clone();
                let images_path = images_dir(state.assets);
                *state.isf = Some(create_isf_state(
                    state.shader_window,
                    isf_path,
                    &images_path,
                ));
                selected_ix = Some(new_ix);
            }
            _ => (),
        }
    }

    // Instantiate the scrollbar for the list.
    if let Some(s) = scrollbar {
        s.set(ui);
    }

    if let Some(isf) = state.isf {
        let (isf_string, isf_color) = match isf.pipeline.isf_err() {
            None => (format!("ISF JSON: OK!"), ui::color::GREEN),
            Some(e) => (format!("ISF JSON: Error\n{}", e), ui::color::RED),
        };
        let (vs_string, vs_color) = match isf.pipeline.vs_err() {
            None => (format!("Vertex Shader: OK!"), ui::color::GREEN),
            Some(e) => (format!("Vertex Shader: Error\n{}", e), ui::color::RED),
        };
        let (fs_string, fs_color) = match isf.pipeline.fs_err() {
            None => (format!("Fragment Shader: OK!"), ui::color::GREEN),
            Some(e) => (format!("Fragment Shader: Error\n{}", e), ui::color::RED),
        };

        status_text(&isf_string)
            .padded_w_of(ids.background_canvas, PAD)
            .middle_of(ids.background_canvas)
            .color(isf_color)
            .down(PAD * 0.8)
            .set(ids.isf_status_text, ui);

        status_text(&vs_string)
            .padded_w_of(ids.background_canvas, PAD)
            .color(vs_color)
            .down(PAD * 0.5)
            .set(ids.vs_status_text, ui);

        status_text(&fs_string)
            .padded_w_of(ids.background_canvas, PAD)
            .color(fs_color)
            .down(PAD * 0.5)
            .set(ids.fs_status_text, ui);

        separator().set(ids.separator0, ui);
    }

    // FPS
    // ------------------------------------------------------------------------

    let fps_avg = format!("FPS AVG: {:.2}", state.fps.avg());
    let fps_min = format!("FPS MIN: {:.2}", state.fps.min());
    let fps_max = format!("FPS MAX: {:.2}", state.fps.max());

    let (r, g, b) = fps_to_rgb(state.fps.avg());
    status_text(&fps_avg)
        .rgb(r, g, b)
        .down(PAD * 0.8)
        .set(ids.fps_avg_text, ui);

    let (r, g, b) = fps_to_rgb(state.fps.min());
    status_text(&fps_min)
        .rgb(r, g, b)
        .down(PAD * 0.5)
        .set(ids.fps_min_text, ui);

    let (r, g, b) = fps_to_rgb(state.fps.max());
    status_text(&fps_max)
        .rgb(r, g, b)
        .down(PAD * 0.5)
        .set(ids.fps_max_text, ui);

    separator().set(ids.separator1, ui);

    // TODO: ISF Data Inputs
}

fn shader_view(_app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    // Encode a command to draw the render pipeline to the frame's texture.
    if let Some(ref isf) = model.isf {
        isf.pipeline.encode_to_frame(&frame, isf.time);
    }

    // Draw some feedback.
    // let win = frame.rect();
    // let draw = app.draw();
    // draw.to_frame(app, &frame).unwrap();
    model.fps.tick();
}

// Find all ISF shaders in the given directory and return them sorted.
fn find_isf_shaders(dir: &Path) -> Vec<PathBuf> {
    assert!(dir.exists());
    assert!(dir.is_dir());
    let mut paths = vec![];
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if ext == Some("fs") || ext == Some("vs") {
            paths.push(path.to_path_buf());
        }
    }
    paths.sort();
    paths
}

// Create a new `IsfState` for the given ISF FS path.
fn create_isf_state(shader_window: &Window, fs_path: PathBuf, images_path: &Path) -> IsfState {
    let device = shader_window.swap_chain_device();
    let vs_path = None;
    let dst_format = Frame::TEXTURE_FORMAT;
    let dst_sample_count = shader_window.msaa_samples();
    let (dst_w, dst_h) = shader_window.inner_size_pixels();
    let desc = wgpu::CommandEncoderDescriptor {
        label: Some("nannou_isf_pipeline_new"),
    };
    let mut encoder = device.create_command_encoder(&desc);
    let pipeline = IsfPipeline::new(
        device,
        &mut encoder,
        vs_path,
        fs_path.clone(),
        dst_format,
        [dst_w, dst_h],
        dst_sample_count,
        images_path,
    );
    shader_window.swap_chain_queue().submit(&[encoder.finish()]);
    let time = Default::default();
    IsfState {
        path: fs_path,
        pipeline,
        time,
    }
}
