use nannou::{rand::thread_rng};
use nannou::{prelude::*, rand::prelude::SliceRandom};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

fn main() {
    nannou::app(model).update(update).run();
}

struct Circle {
    x: f32,
    y: f32,
    radius: f32,
    color: Hsva,
}

struct Settings {
    min_radius: f32,
    max_radius: f32,
    circle_count: usize,
}

struct Model {
    circles: Vec<Circle>,
    settings: Settings,
    window: Entity,
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .size(WIDTH as u32, HEIGHT as u32)
        .view(view)
        .build();

    Model {
        circles: Vec::new(),
        window,
        settings: Settings {
            min_radius: 10.0,
            max_radius: 100.0,
            circle_count: 10,
        },
    }
}

fn update(app: &App, model: &mut Model) {
    let Model {
        window,
        ref mut settings,
        ref mut circles,
        ..
    } = *model;

    let mut egui_ctx = app.egui_for_window(window);
    let ctx = &egui_ctx.get_mut();

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
        if changed {
            *circles = generate_circles(settings);
        }
    });
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();

    draw.background().color(BLACK);

    for circle in model.circles.iter() {
        draw.ellipse()
            .x_y(circle.x, circle.y)
            .radius(circle.radius)
            .color(circle.color);
    }
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
        hsv_from_hex_rgb("#264653"),
        hsv_from_hex_rgb("#2a9d8f"),
        hsv_from_hex_rgb("#e9c46a"),
        hsv_from_hex_rgb("#f4a261"),
        hsv_from_hex_rgb("#e76f51"),
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

fn hsv_from_hex_rgb(color: &str) -> Hsva {
    Srgba::hex(color).unwrap().into()
}
