//! A clone of the `laser_frame_stream.rs` example that allows for configuring laser settings via a
//! UI.

use nannou::geom::Rect;
use nannou::prelude::*;
use nannou_egui::egui::FontId;
use nannou_egui::{self, egui, Egui};
use nannou_laser as laser;
use std::sync::{mpsc, Arc};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    // A handle to the laser API used for spawning streams and detecting DACs.
    laser_api: Arc<laser::Api>,
    // All of the live stream handles.
    laser_streams: Vec<laser::FrameStream<Laser>>,
    // A copy of the state that will live on the laser thread so we can present a GUI.
    laser_model: Laser,
    // A copy of the laser settings so that we can control them with the GUI.
    laser_settings: LaserSettings,
    // For receiving newly detected DACs.
    dac_rx: mpsc::Receiver<laser::DetectedDac>,
    // The UI for control over laser parameters and settings.
    egui: Egui,
}

#[derive(Clone)]
struct Laser {
    draw_mode: DrawMode,
    scale: f32,
    color_profile: RgbProfile,
    point_weight: u32,
    test_pattern: TestPattern,
}

struct LaserSettings {
    point_hz: u32,
    latency_points: u32,
    frame_hz: u32,
    enable_optimisations: bool,
    distance_per_point: f32,
    blank_delay_points: u32,
    radians_per_point: f32,
}

#[derive(Clone, Copy)]
struct RgbProfile {
    rgb: [f32; 3],
}

#[derive(Clone, Copy, PartialEq)]
enum DrawMode {
    Lines,
    Points,
}

// A collection of laser test patterns. We'll toggle between these with the numeric keys.
#[derive(Copy, Clone)]
pub enum TestPattern {
    // A rectangle that outlines the laser's entire field of projection.
    Rectangle,
    // A triangle in the centre of the projection field.
    Triangle,
    // A crosshair in the centre of the projection field that reaches the edges.
    Crosshair,
    // Three vertical lines. One to the far left, one in the centre and one on the right.
    ThreeVerticalLines,
    // A circle whose diameter reaches the edges of the projection field.
    Circle,
    // A spiral that starts from the centre and revolves out towards the edge of the field.
    Spiral,
}

impl Default for Laser {
    fn default() -> Self {
        Laser {
            draw_mode: DrawMode::Lines,
            scale: 1.0,
            point_weight: laser::Point::DEFAULT_LINE_POINT_WEIGHT,
            test_pattern: TestPattern::Rectangle,
            color_profile: Default::default(),
        }
    }
}

impl Default for LaserSettings {
    fn default() -> Self {
        use laser::stream;
        use laser::stream::frame::InterpolationConfig;
        LaserSettings {
            point_hz: stream::DEFAULT_POINT_HZ,
            latency_points: stream::points_per_frame(
                stream::DEFAULT_POINT_HZ,
                stream::DEFAULT_FRAME_HZ,
            ),
            frame_hz: stream::DEFAULT_FRAME_HZ,
            enable_optimisations: true,
            distance_per_point: InterpolationConfig::DEFAULT_DISTANCE_PER_POINT,
            blank_delay_points: InterpolationConfig::DEFAULT_BLANK_DELAY_POINTS,
            radians_per_point: InterpolationConfig::DEFAULT_RADIANS_PER_POINT,
        }
    }
}

impl Default for RgbProfile {
    fn default() -> Self {
        RgbProfile { rgb: [1.0; 3] }
    }
}

fn model(app: &App) -> Model {
    // Create a window to receive keyboard events.
    let w_id = app
        .new_window()
        .size(312, 530)
        .key_pressed(key_pressed)
        .raw_event(raw_window_event)
        .view(view)
        .build()
        .unwrap();

    // Initialise the state that we want to live on the laser thread and spawn the stream.
    let laser_settings = LaserSettings::default();
    let laser_model = Laser::default();

    // TODO Implement `Clone` for `Api` so that we don't have to `Arc` it.
    let laser_api = Arc::new(laser::Api::new());

    // A channel for receiving newly detected DACs.
    let (dac_tx, dac_rx) = mpsc::channel();

    // Spawn a thread for detecting the DACs.
    let laser_api2 = laser_api.clone();
    std::thread::spawn(move || {
        let mut detected = std::collections::HashSet::new();
        for res in laser_api2
            .detect_dacs()
            .expect("failed to start detecting DACs")
        {
            let dac = res.expect("error occurred during DAC detection");
            if detected.insert(dac.id()) {
                println!("{:#?}", dac);
                if dac_tx.send(dac).is_err() {
                    break;
                }
            }
        }
    });

    // We'll use a `Vec` to collect laser streams as they appear.
    let laser_streams = vec![];

    // A user-interface to tweak the settings.
    let window = app.window(w_id).unwrap();
    let egui = Egui::from_window(&window);
    egui.ctx().set_style(style());

    Model {
        laser_api,
        laser_settings,
        laser_model,
        laser_streams,
        dac_rx,
        egui,
    }
}

// Draw lines or points based on the `DrawMode`.
fn add_points<I>(points: I, mode: DrawMode, scale: f32, frame: &mut laser::Frame)
where
    I: IntoIterator,
    I::Item: AsRef<laser::Point>,
{
    let points = points.into_iter().map(|p| {
        let mut p = p.as_ref().clone();
        p.position[0] *= scale;
        p.position[1] *= scale;
        p
    });
    match mode {
        DrawMode::Lines => frame.add_lines(points),
        DrawMode::Points => frame.add_points(points),
    }
}

fn laser(laser: &mut Laser, frame: &mut laser::Frame) {
    // Simple constructor for a lit point.
    let color = laser.color_profile.rgb;
    let weight = laser.point_weight;
    let lit_p = |position| laser::Point {
        position,
        color,
        weight,
    };

    // Retrieve some points to draw based on the pattern.
    match laser.test_pattern {
        TestPattern::Rectangle => {
            let tl = [-1.0, 1.0];
            let tr = [1.0, 1.0];
            let br = [1.0, -1.0];
            let bl = [-1.0, -1.0];
            let positions = [tl, tr, br, bl, tl];
            let points = positions.iter().cloned().map(lit_p);
            add_points(points, laser.draw_mode, laser.scale, frame);
        }

        TestPattern::Triangle => {
            let a = [-0.75, -0.75];
            let b = [0.0, 0.75];
            let c = [0.75, -0.75];
            let positions = [a, b, c, a];
            let points = positions.iter().cloned().map(lit_p);
            add_points(points, laser.draw_mode, laser.scale, frame);
        }

        TestPattern::Crosshair => {
            let xa = [-1.0, 0.0];
            let xb = [1.0, 0.0];
            let ya = [0.0, -1.0];
            let yb = [0.0, 1.0];
            let x = [lit_p(xa), lit_p(xb)];
            let y = [lit_p(ya), lit_p(yb)];
            add_points(&x, laser.draw_mode, laser.scale, frame);
            add_points(&y, laser.draw_mode, laser.scale, frame);
        }

        TestPattern::ThreeVerticalLines => {
            let la = [-1.0, -0.5];
            let lb = [-1.0, 0.5];
            let ma = [0.0, 0.5];
            let mb = [0.0, -0.5];
            let ra = [1.0, -0.5];
            let rb = [1.0, 0.5];
            let l = [lit_p(la), lit_p(lb)];
            let m = [lit_p(ma), lit_p(mb)];
            let r = [lit_p(ra), lit_p(rb)];
            add_points(&l, laser.draw_mode, laser.scale, frame);
            add_points(&m, laser.draw_mode, laser.scale, frame);
            add_points(&r, laser.draw_mode, laser.scale, frame);
        }

        TestPattern::Circle => {
            let n_points = frame.points_per_frame() as usize / 4;
            let rect = Rect::from_w_h(2.0, 2.0);
            let ellipse: Vec<_> = geom::ellipse::Circumference::new(rect, n_points as f32)
                .map(|[x, y]| lit_p([x, y]))
                .collect();
            add_points(&ellipse, laser.draw_mode, laser.scale, frame);
        }

        TestPattern::Spiral => {
            let n_points = frame.points_per_frame() as usize / 2;
            let radius = 1.0;
            let rings = 5.0;
            let points = (0..n_points)
                .map(|i| {
                    let fract = i as f32 / n_points as f32;
                    let mag = fract * radius;
                    let phase = rings * fract * 2.0 * std::f32::consts::PI;
                    let y = mag * -phase.sin();
                    let x = mag * phase.cos();
                    [x, y]
                })
                .map(lit_p)
                .collect::<Vec<_>>();
            add_points(&points, laser.draw_mode, laser.scale, frame);
        }
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, model: &mut Model, update: Update) {
    // First, check for new laser DACs.
    for dac in model.dac_rx.try_recv() {
        println!("Detected DAC {:?}!", dac.id());
        let stream = model
            .laser_api
            .new_frame_stream(model.laser_model.clone(), laser)
            .detected_dac(dac)
            .build()
            .expect("failed to establish stream with newly detected DAC");
        model.laser_streams.push(stream);
    }

    // Check if any streams have dropped out (e.g network issues, DAC turned off) and attempt to
    // start them again.
    let mut dropped = vec![];
    for (i, stream) in model.laser_streams.iter().enumerate() {
        if stream.is_closed() {
            dropped.push(i);
        }
    }
    for i in dropped.into_iter().rev() {
        let stream = model.laser_streams.remove(i);
        let dac = stream
            .dac()
            .expect("`dac` returned `None` even though one was specified during stream creation");
        let res = stream
            .close()
            .expect("stream was unexpectedly already closed from another stream handle")
            .expect("failed to join stream thread");
        if let Err(err) = res {
            eprintln!("Stream closed due to an error: {}", err);
        }
        println!("attempting to restart stream with DAC {:?}", dac.id());
        match model
            .laser_api
            .new_frame_stream(model.laser_model.clone(), laser)
            .detected_dac(dac)
            .build()
        {
            Err(err) => eprintln!("failed to restart stream: {}", err),
            Ok(stream) => model.laser_streams.push(stream),
        }
    }

    // Update the GUI.
    let Model {
        ref mut egui,
        ref laser_streams,
        ref mut laser_model,
        ref mut laser_settings,
        ..
    } = *model;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    // The timeline area.
    egui::containers::CentralPanel::default().show(&ctx, |ui| {
        fn grid_min_col_width(ui: &egui::Ui, n_options: usize) -> f32 {
            let gap_space = ui.spacing().item_spacing.x * (n_options as f32 - 1.0);
            let grid_w = ui.available_width();
            (grid_w - gap_space) / n_options as f32
        }

        ui.heading("Laser Points");

        let col_w = grid_min_col_width(ui, 2);
        egui::Grid::new("Mode")
            .min_col_width(col_w)
            .max_col_width(col_w)
            .show(ui, |ui| {
                use DrawMode::{Lines, Points};
                let mut changed = false;
                ui.vertical_centered_justified(|ui| {
                    changed |= ui
                        .selectable_value(&mut laser_model.draw_mode, Lines, "LINES")
                        .changed();
                });
                ui.vertical_centered_justified(|ui| {
                    changed |= ui
                        .selectable_value(&mut laser_model.draw_mode, Points, "POINTS")
                        .changed();
                });
                if changed {
                    let mode = laser_model.draw_mode;
                    for stream in laser_streams {
                        stream.send(move |laser| laser.draw_mode = mode).ok();
                    }
                }
            });

        if ui
            .add(egui::Slider::new(&mut laser_model.scale, 0.0..=1.0).text("Scale"))
            .changed()
        {
            let scale = laser_model.scale;
            for stream in laser_streams {
                stream.send(move |laser| laser.scale = scale).ok();
            }
        }
        if ui
            .add(egui::Slider::new(&mut laser_model.point_weight, 0..=128).text("Point Weight"))
            .changed()
        {
            let scale = laser_model.scale;
            for stream in laser_streams {
                stream.send(move |laser| laser.scale = scale).ok();
            }
        }

        ui.separator();

        ui.heading("Laser Settings");

        if ui
            .add(egui::Slider::new(&mut laser_settings.point_hz, 1_000..=10_000).text("DAC PPS"))
            .changed()
        {
            let hz = laser_settings.point_hz;
            for stream in laser_streams {
                stream.set_point_hz(hz).ok();
            }
        }
        if ui
            .add(egui::Slider::new(&mut laser_settings.latency_points, 10..=1_500).text("Latency"))
            .changed()
        {
            let latency = laser_settings.latency_points;
            for stream in laser_streams {
                stream.set_latency_points(latency).ok();
            }
        }
        if ui
            .add(egui::Slider::new(&mut laser_settings.frame_hz, 1..=120).text("Target FPS"))
            .changed()
        {
            let hz = laser_settings.frame_hz;
            for stream in laser_streams {
                stream.set_frame_hz(hz).ok();
            }
        }

        ui.separator();

        ui.heading("Laser Path Interpolation");

        if ui
            .checkbox(&mut laser_settings.enable_optimisations, "Optimize Path")
            .changed()
        {
            for stream in laser_streams {
                stream
                    .enable_optimisations(laser_settings.enable_optimisations)
                    .ok();
            }
        }
        if ui
            .add(
                egui::Slider::new(&mut laser_settings.distance_per_point, 0.01..=1.0)
                    .text("Distance Per Point"),
            )
            .changed()
        {
            let distance = laser_settings.distance_per_point;
            for stream in laser_streams {
                stream.set_distance_per_point(distance).ok();
            }
        }
        if ui
            .add(
                egui::Slider::new(&mut laser_settings.blank_delay_points, 0..=32)
                    .text("Blank Delay (Points)"),
            )
            .changed()
        {
            let delay = laser_settings.blank_delay_points;
            for stream in laser_streams {
                stream.set_blank_delay_points(delay).ok();
            }
        }
        let mut degrees = rad_to_deg(laser_settings.radians_per_point);
        if ui
            .add(egui::Slider::new(&mut degrees, 1.0..=180.0).text("Degrees Per Point"))
            .changed()
        {
            let radians = deg_to_rad(degrees);
            laser_settings.radians_per_point = radians;
            for stream in laser_streams {
                stream.set_radians_per_point(radians).ok();
            }
        }

        ui.separator();

        ui.heading("Color Profile");

        if ui
            .color_edit_button_rgb(&mut laser_model.color_profile.rgb)
            .changed()
        {
            let rgb = laser_model.color_profile.rgb;
            for stream in laser_streams {
                stream.send(move |model| model.color_profile.rgb = rgb).ok();
            }
        }
    });
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    // Send a new pattern to the laser on keys 1, 2, 3 and 4.
    let new_pattern = match key {
        Key::Key1 => TestPattern::Rectangle,
        Key::Key2 => TestPattern::Triangle,
        Key::Key3 => TestPattern::Crosshair,
        Key::Key4 => TestPattern::ThreeVerticalLines,
        Key::Key5 => TestPattern::Circle,
        Key::Key6 => TestPattern::Spiral,
        _ => return,
    };
    for stream in &model.laser_streams {
        stream
            .send(move |laser| laser.test_pattern = new_pattern)
            .ok();
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    model.egui.draw_to_frame(&frame).unwrap();
}

// The following functions are some custom styling preferences in an attempt to improve on the
// default egui theming.

fn style() -> egui::Style {
    let mut style = egui::Style::default();
    style.spacing = egui::style::Spacing {
        item_spacing: egui::Vec2::splat(8.0),
        window_margin: egui::Margin {
            left: 6.0,
            right: 6.0,
            top: 6.0,
            bottom: 6.0,
        },
        button_padding: egui::Vec2::new(4.0, 2.0),
        interact_size: egui::Vec2::new(56.0, 24.0),
        indent: 10.0,
        icon_width: 20.0,
        icon_spacing: 1.0,
        ..style.spacing
    };
    style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::WHITE;
    style.visuals.extreme_bg_color = egui::Color32::from_gray(12);
    style.visuals.faint_bg_color = egui::Color32::from_gray(24);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_gray(36);
    style.visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::BLACK;
    style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::WHITE;
    style.text_styles = [
        (
            egui::TextStyle::Small,
            FontId::new(13.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Body,
            FontId::new(16.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Button,
            FontId::new(16.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Heading,
            FontId::new(20.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Monospace,
            FontId::new(14.0, egui::FontFamily::Monospace),
        ),
    ]
    .into();
    style
}
