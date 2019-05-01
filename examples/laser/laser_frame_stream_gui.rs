//! A clone of the `laser_frame_stream.rs` example that allows for configuring laser settings via a
//! UI.

use nannou::geom::Rect;
use nannou::prelude::*;
use nannou::ui::prelude::*;
use std::sync::{mpsc, Arc};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    // A handle to the laser API used for spawning streams and detecting DACs.
    laser_api: Arc<lasy::Lasy>,
    // All of the live stream handles.
    laser_streams: Vec<lasy::FrameStream<Laser>>,
    // A copy of the state that will live on the laser thread so we can present a GUI.
    laser_model: Laser,
    // A copy of the laser settings so that we can control them with the GUI.
    laser_settings: LaserSettings,
    // For receiving newly detected DACs.
    dac_rx: mpsc::Receiver<lasy::DetectedDac>,
    // The UI for control over laser parameters and settings.
    ui: Ui,
    // The unique ID for each UI widget.
    ids: Ids,
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
    distance_per_point: f32,
    blank_delay_points: u32,
    radians_per_point: f32,
}

#[derive(Clone, Copy)]
struct RgbProfile {
    red: f32,
    green: f32,
    blue: f32,
}

#[derive(Clone, Copy)]
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

struct Ids {
    background_canvas: widget::Id,
    laser_points_text: widget::Id,
    draw_mode_lines_button: widget::Id,
    draw_mode_points_button: widget::Id,
    point_weight_slider: widget::Id,
    scale_slider: widget::Id,
    laser_settings_text: widget::Id,
    point_hz_slider: widget::Id,
    latency_points_slider: widget::Id,
    frame_hz_slider: widget::Id,
    laser_path_interpolation_text: widget::Id,
    distance_per_point_slider: widget::Id,
    blank_delay_points_slider: widget::Id,
    radians_per_point_slider: widget::Id,
    color_profile_text: widget::Id,
    red_slider: widget::Id,
    green_slider: widget::Id,
    blue_slider: widget::Id,
}

impl Default for Laser {
    fn default() -> Self {
        Laser {
            draw_mode: DrawMode::Lines,
            scale: 1.0,
            point_weight: lasy::Point::DEFAULT_LINE_POINT_WEIGHT,
            test_pattern: TestPattern::Rectangle,
            color_profile: Default::default(),
        }
    }
}

impl Default for LaserSettings {
    fn default() -> Self {
        use lasy::stream;
        use lasy::stream::frame::opt::InterpolationConfig;
        LaserSettings {
            point_hz: stream::DEFAULT_POINT_HZ,
            latency_points: stream::points_per_frame(
                stream::DEFAULT_POINT_HZ,
                stream::DEFAULT_FRAME_HZ,
            ),
            frame_hz: stream::DEFAULT_FRAME_HZ,
            distance_per_point: InterpolationConfig::DEFAULT_DISTANCE_PER_POINT,
            blank_delay_points: InterpolationConfig::DEFAULT_BLANK_DELAY_POINTS,
            radians_per_point: InterpolationConfig::DEFAULT_RADIANS_PER_POINT,
        }
    }
}

impl Default for RgbProfile {
    fn default() -> Self {
        RgbProfile {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
        }
    }
}

fn model(app: &App) -> Model {
    // Create a window to receive keyboard events.
    app.new_window()
        .with_dimensions(240, 620)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the state that we want to live on the laser thread and spawn the stream.
    let laser_settings = LaserSettings::default();
    let laser_model = Laser::default();

    // TODO Implement `Clone` for `Lasy` so that we don't have to `Arc` it.
    let laser_api = Arc::new(lasy::Lasy::new());

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
    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids {
        background_canvas: ui.generate_widget_id(),
        laser_points_text: ui.generate_widget_id(),
        draw_mode_lines_button: ui.generate_widget_id(),
        draw_mode_points_button: ui.generate_widget_id(),
        point_weight_slider: ui.generate_widget_id(),
        scale_slider: ui.generate_widget_id(),
        laser_settings_text: ui.generate_widget_id(),
        point_hz_slider: ui.generate_widget_id(),
        latency_points_slider: ui.generate_widget_id(),
        frame_hz_slider: ui.generate_widget_id(),
        laser_path_interpolation_text: ui.generate_widget_id(),
        distance_per_point_slider: ui.generate_widget_id(),
        blank_delay_points_slider: ui.generate_widget_id(),
        radians_per_point_slider: ui.generate_widget_id(),
        color_profile_text: ui.generate_widget_id(),
        red_slider: ui.generate_widget_id(),
        green_slider: ui.generate_widget_id(),
        blue_slider: ui.generate_widget_id(),
    };

    Model {
        laser_api,
        laser_settings,
        laser_model,
        laser_streams,
        dac_rx,
        ui,
        ids,
    }
}

// Draw lines or points based on the `DrawMode`.
fn add_points<I>(points: I, mode: DrawMode, scale: f32, frame: &mut lasy::Frame)
where
    I: IntoIterator,
    I::Item: AsRef<lasy::Point>,
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

fn laser(laser: &mut Laser, frame: &mut lasy::Frame) {
    // Simple constructor for a lit point.
    let color = [
        laser.color_profile.red,
        laser.color_profile.green,
        laser.color_profile.blue,
    ];
    let weight = laser.point_weight;
    let lit_p = |position| lasy::Point {
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
            let ellipse: Vec<_> = geom::ellipse::Circumference::new(rect, n_points)
                .map(|p| lit_p([p.x, p.y]))
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

fn update(_app: &App, model: &mut Model, _update: Update) {
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

    // Calling `set_widgets` allows us to instantiate some widgets.
    let ui = &mut model.ui.set_widgets();

    fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, min, max)
            .down(5.0)
            .w_h(200.0, 30.0)
            .label_font_size(12)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }

    fn button() -> widget::Button<'static, widget::button::Flat> {
        widget::Button::new()
            .w_h(95.0, 30.0)
            .label_font_size(12)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }

    widget::Canvas::new()
        .rgb(0.011, 0.013, 0.017)
        .set(model.ids.background_canvas, ui);

    widget::Text::new("Laser Points")
        .color(color::WHITE)
        .top_left_with_margin(20.0)
        .set(model.ids.laser_points_text, ui);

    // Button colours.
    let (lines_color, points_color) = match model.laser_model.draw_mode {
        DrawMode::Lines => (color::BLUE, color::BLACK),
        DrawMode::Points => (color::BLACK, color::BLUE),
    };

    for _click in button()
        .label("LINES")
        .down(20.0)
        .color(lines_color)
        .set(model.ids.draw_mode_lines_button, ui)
    {
        let mode = DrawMode::Lines;
        model.laser_model.draw_mode = mode;
        for stream in &model.laser_streams {
            stream.send(move |laser| laser.draw_mode = mode).ok();
        }
    }

    for _click in button()
        .label("POINTS")
        .right(10.0)
        .color(points_color)
        .set(model.ids.draw_mode_points_button, ui)
    {
        let mode = DrawMode::Points;
        model.laser_model.draw_mode = mode;
        for stream in &model.laser_streams {
            stream.send(move |laser| laser.draw_mode = mode).ok();
        }
    }

    let label = format!("Scale: {}", model.laser_model.scale);
    for value in slider(model.laser_model.scale as _, 0.0, 1.0)
        .down_from(model.ids.draw_mode_lines_button, 10.0)
        .label(&label)
        .set(model.ids.scale_slider, ui)
    {
        model.laser_model.scale = value as _;
        for stream in &model.laser_streams {
            stream.send(move |laser| laser.scale = value as _).ok();
        }
    }

    let label = format!("Point Weight: {}", model.laser_model.point_weight);
    for value in slider(model.laser_model.point_weight as _, 0.0, 128.0)
        .label(&label)
        .set(model.ids.point_weight_slider, ui)
    {
        model.laser_model.point_weight = value as _;
        for stream in &model.laser_streams {
            stream
                .send(move |laser| laser.point_weight = value as _)
                .ok();
        }
    }

    widget::Text::new("Laser Settings")
        .color(color::WHITE)
        .down(20.0)
        .set(model.ids.laser_settings_text, ui);

    let label = format!("DAC PPS: {}", model.laser_settings.point_hz);
    for value in slider(model.laser_settings.point_hz as _, 1_000.0, 10_000.0)
        .down(20.0)
        .label(&label)
        .set(model.ids.point_hz_slider, ui)
    {
        model.laser_settings.point_hz = value as _;
        for stream in &model.laser_streams {
            stream.set_point_hz(value as _).ok();
        }
    }

    let label = format!("Latency: {} points", model.laser_settings.latency_points);
    for value in slider(model.laser_settings.latency_points as _, 10.0, 1_500.0)
        .label(&label)
        .set(model.ids.latency_points_slider, ui)
    {
        model.laser_settings.latency_points = value as _;
        for stream in &model.laser_streams {
            stream.set_latency_points(value as _).ok();
        }
    }

    let label = format!("Target FPS: {}", model.laser_settings.frame_hz);
    for value in slider(model.laser_settings.frame_hz as _, 1.0, 120.0)
        .label(&label)
        .set(model.ids.frame_hz_slider, ui)
    {
        model.laser_settings.frame_hz = value as _;
        for stream in &model.laser_streams {
            stream.set_frame_hz(value as _).ok();
        }
    }

    widget::Text::new("Laser Path Interpolation")
        .down(20.0)
        .color(color::WHITE)
        .font_size(16)
        .set(model.ids.laser_path_interpolation_text, ui);

    let label = format!(
        "Distance per point: {:.2}",
        model.laser_settings.distance_per_point
    );
    for value in slider(model.laser_settings.distance_per_point, 0.01, 1.0)
        .down(20.0)
        .label(&label)
        .set(model.ids.distance_per_point_slider, ui)
    {
        model.laser_settings.distance_per_point = value;
        for stream in &model.laser_streams {
            stream.set_distance_per_point(value).ok();
        }
    }

    let label = format!(
        "Blank delay: {} points",
        model.laser_settings.blank_delay_points
    );
    for value in slider(model.laser_settings.blank_delay_points as _, 0.0, 32.0)
        .label(&label)
        .set(model.ids.blank_delay_points_slider, ui)
    {
        model.laser_settings.blank_delay_points = value as _;
        for stream in &model.laser_streams {
            stream.set_blank_delay_points(value as _).ok();
        }
    }

    let degrees_per_point = rad_to_deg(model.laser_settings.radians_per_point);
    let label = format!("Degrees per point (inertia): {:.2}", degrees_per_point);
    for value in slider(degrees_per_point, 1.0, 180.0)
        .label(&label)
        .set(model.ids.radians_per_point_slider, ui)
    {
        let radians = deg_to_rad(value);
        model.laser_settings.radians_per_point = radians;
        for stream in &model.laser_streams {
            stream.set_radians_per_point(radians).ok();
        }
    }

    widget::Text::new("Color Profile")
        .down(20.0)
        .color(color::WHITE)
        .font_size(16)
        .set(model.ids.color_profile_text, ui);

    for value in slider(model.laser_model.color_profile.red, 0.0, 1.0)
        .down(20.0)
        .color(color::RED)
        .set(model.ids.red_slider, ui)
    {
        model.laser_model.color_profile.red = value;
        for stream in &model.laser_streams {
            stream
                .send(move |model| model.color_profile.red = value)
                .ok();
        }
    }

    for value in slider(model.laser_model.color_profile.green, 0.0, 1.0)
        .color(color::GREEN)
        .set(model.ids.green_slider, ui)
    {
        model.laser_model.color_profile.green = value;
        for stream in &model.laser_streams {
            stream
                .send(move |model| model.color_profile.green = value)
                .ok();
        }
    }

    for value in slider(model.laser_model.color_profile.blue, 0.0, 1.0)
        .color(color::BLUE)
        .set(model.ids.blue_slider, ui)
    {
        model.laser_model.color_profile.blue = value;
        for stream in &model.laser_streams {
            stream
                .send(move |model| model.color_profile.blue = value)
                .ok();
        }
    }
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

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    model.ui.draw_to_frame_if_changed(app, &frame).unwrap();
    frame
}
