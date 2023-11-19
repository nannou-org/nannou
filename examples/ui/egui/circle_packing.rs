use nannou::{color::rgb_u32, rand::thread_rng};
use nannou::{prelude::*, rand::prelude::SliceRandom};
use nannou_egui::{self, egui, Egui};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

fn main() {
    nannou::app(model).update(update).run();
}

struct Circle {
    x: f32,
    y: f32,
    radius: f32,
    color: Hsv,
}

struct Settings {
    min_radius: f32,
    max_radius: f32,
    circle_count: usize,
}

struct Model {
    circles: Vec<Circle>,
    settings: Settings,
    egui: Egui,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(WIDTH as u32, HEIGHT as u32)
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    Model {
        circles: Vec::new(),
        egui,
        settings: Settings {
            min_radius: 10.0,
            max_radius: 100.0,
            circle_count: 10,
        },
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let Model {
        egui,
        settings,
        circles,
        ..
    } = model;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Workshop window").show(&ctx, |ui| {
        let mut changed = false;
        changed |= ui
            .add(egui::Slider::new(&mut settings.min_radius, 0.0..=20.0).text("min radius"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut settings.max_radius, 0.0..=200.0).text("max radius"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut settings.circle_count, 0..=2000).text("circle count"))
            .changed();
        changed |= ui.button("Generate").clicked();
        if !changed {
            *circles = generate_circles(settings);
        }
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);

    for circle in model.circles.iter() {
        draw.ellipse()
            .x_y(circle.x, circle.y)
            .radius(circle.radius)
            .color(circle.color);
    }

    draw.to_frame(app, &frame).unwrap();

    model.egui.draw_to_frame(&frame).unwrap();
}

fn intersects(circle: &Circle, circles: &Vec<Circle>) -> bool {
    for other in circles.iter() {
        let dist: f32 =
            ((other.x - circle.x).pow(2) as f32 + (other.y - circle.y).pow(2) as f32).sqrt();
        if dist < circle.radius + other.radius {
            return true;
        }
    }
    false
}

fn generate_circles(settings: &mut Settings) -> Vec<Circle> {
    let colors = [
        hsv_from_hex_rgb(0x264653),
        hsv_from_hex_rgb(0x2a9d8f),
        hsv_from_hex_rgb(0xe9c46a),
        hsv_from_hex_rgb(0xf4a261),
        hsv_from_hex_rgb(0xe76f51),
    ];

    let mut circles = Vec::new();

    let mut rng = thread_rng();

    let mut loops = 0;
    loop {
        let x = random_range(-WIDTH / 2.0, WIDTH / 2.0);
        let y = random_range(-HEIGHT / 2.0, HEIGHT / 2.0);
        let radius = random_range(settings.min_radius, settings.max_radius);
        let color = *colors.choose(&mut rng).unwrap();
        let mut circle = Circle {
            x,
            y,
            radius,
            color,
        };

        loops += 1;
        if loops > 20000 {
            break;
        }

        if intersects(&circle, &circles) {
            continue;
        }

        let mut prev_radius = circle.radius;
        while !intersects(&circle, &circles) {
            // Grow the circle
            prev_radius = circle.radius;
            circle.radius += 10.0;

            if circle.radius >= settings.max_radius {
                break;
            }
        }
        circle.radius = prev_radius;

        circles.push(circle);

        if circles.len() >= settings.circle_count {
            break;
        }
    }

    circles
}

fn hsv_from_hex_rgb(color: u32) -> Hsv {
    let color = rgb_u32(color);
    rgba(
        color.red as f32 / 255.0,
        color.green as f32 / 255.0,
        color.blue as f32 / 255.0,
        1.0,
    )
    .into()
}
