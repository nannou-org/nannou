use nannou::prelude::*;
use std::f32::consts::{PI, TAU};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 820;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DebugMode {
    Shaded,
    DirtyBricks,
    BrickResidency,
    Distance,
    Normals,
}

impl DebugMode {
    fn sdf(self) -> SdfDebugView {
        match self {
            Self::Shaded => SdfDebugView::None,
            Self::DirtyBricks => SdfDebugView::DirtyBricks,
            Self::BrickResidency => SdfDebugView::BrickResidency,
            Self::Distance => SdfDebugView::Distance,
            Self::Normals => SdfDebugView::Normals,
        }
    }
}

struct Settings {
    animate: bool,
    temperature: f32,
    light_intensity: f32,
    ambient: f32,
    diffuse: f32,
    smoothness: f32,
    core_radius: f32,
    tunnel_radius: f32,
    tower_height: f32,
    satellite_count: u32,
    camera_distance: f32,
    camera_orbit_speed: f32,
    debug: DebugMode,
}

struct Model {
    window: Entity,
    settings: Settings,
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .title("Advanced SDF Reactor")
        .size(WIDTH, HEIGHT)
        .view(view)
        .build();

    Model {
        window,
        settings: Settings {
            animate: true,
            temperature: 5200.0,
            light_intensity: 1.7,
            ambient: 0.18,
            diffuse: 1.05,
            smoothness: 18.0,
            core_radius: 72.0,
            tunnel_radius: 18.0,
            tower_height: 92.0,
            satellite_count: 14,
            camera_distance: 560.0,
            camera_orbit_speed: 0.08,
            debug: DebugMode::Shaded,
        },
    }
}

fn update(app: &App, model: &mut Model) {
    let ctx = app.egui_for_window(model.window);
    egui::Window::new("SDF Reactor")
        .default_pos(egui::pos2(16.0, 16.0))
        .default_width(280.0)
        .show(&ctx, |ui| {
            ui.checkbox(&mut model.settings.animate, "Animate");
            ui.add(
                egui::Slider::new(&mut model.settings.temperature, 900.0..=12_000.0)
                    .text("Blackbody K"),
            );
            ui.add(egui::Slider::new(&mut model.settings.light_intensity, 0.2..=4.0).text("Light"));
            ui.add(egui::Slider::new(&mut model.settings.ambient, 0.0..=0.7).text("Ambient"));
            ui.add(egui::Slider::new(&mut model.settings.diffuse, 0.0..=2.0).text("Diffuse"));
            ui.separator();
            ui.add(egui::Slider::new(&mut model.settings.core_radius, 44.0..=110.0).text("Core"));
            ui.add(
                egui::Slider::new(&mut model.settings.tunnel_radius, 4.0..=40.0).text("Tunnels"),
            );
            ui.add(egui::Slider::new(&mut model.settings.smoothness, 0.0..=40.0).text("Blend"));
            ui.add(
                egui::Slider::new(&mut model.settings.tower_height, 35.0..=150.0).text("Towers"),
            );
            ui.add(
                egui::Slider::new(&mut model.settings.satellite_count, 6..=24).text("Satellites"),
            );
            ui.separator();
            ui.add(
                egui::Slider::new(&mut model.settings.camera_distance, 380.0..=760.0)
                    .text("Camera"),
            );
            ui.add(
                egui::Slider::new(&mut model.settings.camera_orbit_speed, -0.25..=0.25)
                    .text("Orbit"),
            );
            egui::ComboBox::from_label("View")
                .selected_text(match model.settings.debug {
                    DebugMode::Shaded => "Shaded",
                    DebugMode::DirtyBricks => "Dirty bricks",
                    DebugMode::BrickResidency => "Brick residency",
                    DebugMode::Distance => "Distance",
                    DebugMode::Normals => "Normals",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut model.settings.debug, DebugMode::Shaded, "Shaded");
                    ui.selectable_value(
                        &mut model.settings.debug,
                        DebugMode::DirtyBricks,
                        "Dirty bricks",
                    );
                    ui.selectable_value(
                        &mut model.settings.debug,
                        DebugMode::BrickResidency,
                        "Brick residency",
                    );
                    ui.selectable_value(&mut model.settings.debug, DebugMode::Distance, "Distance");
                    ui.selectable_value(&mut model.settings.debug, DebugMode::Normals, "Normals");
                });

            let status = app.sdf_for_window(model.window).status();
            ui.separator();
            ui.label(format!("Dirty bricks: {}", status.dirty_bricks));
            ui.label(format!(
                "Resident bricks: {} / {}",
                status.resident_bricks, status.atlas_capacity
            ));
        });

    build_scene(app, &model.settings);
}

fn build_scene(app: &App, settings: &Settings) {
    let sdf = app.sdf();
    let time = scene_time(app, settings);
    let heat = smootherstep(900.0, 12_000.0, settings.temperature);
    let core_color = blackbody_color(settings.temperature + 1500.0);
    let lamp_color = blackbody_color(settings.temperature + 3200.0);
    let metal_color = Color::srgb(0.08 + heat * 0.08, 0.11 + heat * 0.06, 0.15 + heat * 0.08);
    let cool_color = Color::srgb(0.08, 0.24 + heat * 0.08, 0.42 + heat * 0.24);

    sdf.configure()
        .bounds(SdfBounds::from_center_size(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(640.0, 390.0, 640.0),
        ))
        .voxel_size(4.0)
        .brick_size(8)
        .narrow_band(16.0)
        .atlas_capacity(8192)
        .update_budget(SdfUpdateBudget::MaxBricksPerFrame(4096));

    sdf.transaction(|s| {
        s.rounded_cuboid()
            .key("foundation")
            .w_h_d(540.0, 24.0, 540.0)
            .roundness(12.0)
            .y(-112.0)
            .color(metal_color);

        s.rounded_cuboid()
            .key("lower_deck")
            .w_h_d(390.0, 18.0, 390.0)
            .roundness(10.0)
            .yaw(PI * 0.25)
            .y(-82.0)
            .smooth_union(settings.smoothness)
            .color(Color::srgb(0.11, 0.14, 0.18));

        s.sphere()
            .key("reactor_core")
            .radius(settings.core_radius)
            .smooth_union(settings.smoothness)
            .color(core_color);

        s.torus()
            .key("equator_ring")
            .major_radius(settings.core_radius * 1.38)
            .minor_radius(8.0 + heat * 10.0)
            .yaw(time * 0.24)
            .smooth_union(settings.smoothness * 0.7)
            .color(lamp_color);

        s.torus()
            .key("vertical_ring_a")
            .major_radius(settings.core_radius * 1.18)
            .minor_radius(5.5 + heat * 5.0)
            .pitch(PI * 0.5)
            .roll(time * 0.18)
            .smooth_union(settings.smoothness * 0.6)
            .color(cool_color);

        s.torus()
            .key("vertical_ring_b")
            .major_radius(settings.core_radius * 1.08)
            .minor_radius(4.5 + heat * 4.0)
            .pitch(PI * 0.5)
            .yaw(PI * 0.5)
            .roll(-time * 0.16)
            .smooth_union(settings.smoothness * 0.6)
            .color(blackbody_color(settings.temperature + 600.0));

        for (key, from, to) in [
            (
                "cut_x",
                Vec3::new(-170.0, 0.0, 0.0),
                Vec3::new(170.0, 0.0, 0.0),
            ),
            (
                "cut_y",
                Vec3::new(0.0, -145.0, 0.0),
                Vec3::new(0.0, 145.0, 0.0),
            ),
            (
                "cut_z",
                Vec3::new(0.0, 0.0, -170.0),
                Vec3::new(0.0, 0.0, 170.0),
            ),
        ] {
            s.capsule()
                .key(key)
                .from_to(from, to)
                .radius(settings.tunnel_radius)
                .smooth_subtract(settings.smoothness * 0.8);
        }

        for i in 0..10 {
            let a = i as f32 / 10.0 * TAU + time * 0.12;
            let inner = Vec3::new(a.cos() * 88.0, -54.0, a.sin() * 88.0);
            let outer = Vec3::new(
                a.cos() * 250.0,
                -72.0 + (a * 3.0).sin() * 9.0,
                a.sin() * 250.0,
            );
            s.capsule()
                .key(format!("thermal_spoke_{i}"))
                .from_to(inner, outer)
                .radius(6.0 + heat * 3.0)
                .smooth_union(8.0)
                .color(blackbody_color(settings.temperature + i as f32 * 220.0));
        }

        for ix in -2..=2 {
            for iz in -2..=2 {
                if ix == 0 && iz == 0 {
                    continue;
                }
                let x = ix as f32 * 106.0;
                let z = iz as f32 * 106.0;
                let distance = Vec2::new(ix as f32, iz as f32).length();
                let height = settings.tower_height * (1.08 - distance * 0.08).max(0.45);
                let y = -82.0 + height * 0.5;
                let spin = time * (0.1 + distance * 0.03);
                s.rounded_cuboid()
                    .key(format!("tower_{ix}_{iz}"))
                    .w_h_d(24.0, height, 24.0)
                    .roundness(5.0)
                    .xyz(Vec3::new(x, y, z))
                    .yaw(spin)
                    .color(metal_color);
                s.sphere()
                    .key(format!("tower_lamp_{ix}_{iz}"))
                    .radius(12.0 + heat * 8.0)
                    .xyz(Vec3::new(x, y + height * 0.5 + 18.0, z))
                    .smooth_union(settings.smoothness * 0.35)
                    .color(blackbody_color(
                        settings.temperature + 1800.0 - distance * 160.0,
                    ));
            }
        }

        let satellites = settings.satellite_count.max(1);
        for i in 0..satellites {
            let phase = i as f32 / satellites as f32;
            let a = phase * TAU + time * 0.38;
            let lift = (phase * TAU * 3.0 + time).sin() * 42.0;
            let radius = 155.0 + (phase * TAU * 5.0).sin() * 28.0;
            let pos = Vec3::new(a.cos() * radius, 38.0 + lift, a.sin() * radius);
            s.ellipsoid()
                .key(format!("satellite_{i}"))
                .radii(Vec3::new(
                    11.0 + heat * 5.0,
                    18.0 + heat * 8.0,
                    11.0 + heat * 5.0,
                ))
                .xyz(pos)
                .yaw(-a)
                .smooth_union(5.0)
                .color(blackbody_color(
                    settings.temperature + 2400.0 + phase * 900.0,
                ));
        }
    });
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let time = scene_time(app, &model.settings);
    let camera_angle = time * model.settings.camera_orbit_speed + 0.72;
    let camera = SdfCamera {
        position: Vec3::new(
            camera_angle.sin() * model.settings.camera_distance,
            172.0,
            camera_angle.cos() * model.settings.camera_distance,
        ),
        target: Vec3::new(0.0, -12.0, 0.0),
        fov_y_radians: 42.0_f32.to_radians(),
        ..Default::default()
    };
    let light_yaw = time * 0.21 + 0.9;
    let light_from = Vec3::new(light_yaw.cos() * 0.7, 0.9, light_yaw.sin() * 0.7).normalize();
    let light_color = blackbody_linear(model.settings.temperature) * model.settings.light_intensity;

    draw.background().color(Color::srgb(0.005, 0.007, 0.011));
    draw.sdf(&app.sdf_for_window(model.window))
        .camera(camera)
        .lighting(SdfLighting {
            direction: -light_from,
            color: light_color,
            ambient: model.settings.ambient,
            diffuse: model.settings.diffuse,
        })
        .debug(model.settings.debug.sdf())
        .max_steps(768)
        .hit_epsilon(1.5)
        .normal_epsilon(4.0)
        .max_distance(1200.0);
}

fn scene_time(app: &App, settings: &Settings) -> f32 {
    if settings.animate { app.time() } else { 0.0 }
}

fn blackbody_color(kelvin: f32) -> Color {
    let rgb = blackbody_srgb(kelvin);
    Color::srgb(rgb.x, rgb.y, rgb.z)
}

fn blackbody_linear(kelvin: f32) -> Vec3 {
    srgb_to_linear(blackbody_srgb(kelvin))
}

fn blackbody_srgb(kelvin: f32) -> Vec3 {
    let t = (kelvin.clamp(900.0, 40_000.0) / 100.0).max(1.0);
    let red = if t <= 66.0 {
        255.0
    } else {
        329.698_73 * (t - 60.0).powf(-0.133_204_76)
    };
    let green = if t <= 66.0 {
        99.470_8 * t.ln() - 161.119_57
    } else {
        288.122_16 * (t - 60.0).powf(-0.075_514_846)
    };
    let blue = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_73 * (t - 10.0).ln() - 305.044_8
    };
    Vec3::new(red, green, blue).clamp(Vec3::ZERO, Vec3::splat(255.0)) / 255.0
}

fn srgb_to_linear(rgb: Vec3) -> Vec3 {
    Vec3::new(
        srgb_channel_to_linear(rgb.x),
        srgb_channel_to_linear(rgb.y),
        srgb_channel_to_linear(rgb.z),
    )
}

fn srgb_channel_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
