use nannou::{prelude::*, sdf};
use std::f32::consts::PI;

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
    width: f32,
    depth: f32,
    amplitude: f32,
    base_height: f32,
    floor_depth: f32,
    seed: u32,
    noise_scale: f32,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    ridge: f32,
    offset_x: f32,
    offset_z: f32,
    camera_distance: f32,
    camera_height: f32,
    camera_yaw: f32,
    fov_degrees: f32,
    debug: DebugMode,
}

struct Model {
    window: Entity,
    settings: Settings,
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .title("SDF Procedural Terrain")
        .size(WIDTH, HEIGHT)
        .view(view)
        .build();

    Model {
        window,
        settings: Settings {
            width: 420.0,
            depth: 420.0,
            amplitude: 48.0,
            base_height: 0.0,
            floor_depth: 18.0,
            seed: 7,
            noise_scale: 0.016,
            octaves: 5,
            lacunarity: 2.05,
            gain: 0.48,
            ridge: 0.25,
            offset_x: 0.0,
            offset_z: 0.0,
            camera_distance: 520.0,
            camera_height: 430.0,
            camera_yaw: 0.72,
            fov_degrees: 38.0,
            debug: DebugMode::Shaded,
        },
    }
}

fn update(app: &App, model: &mut Model) {
    let ctx = app.egui_for_window(model.window);
    egui::Window::new("Terrain")
        .default_pos(egui::pos2(16.0, 16.0))
        .default_width(300.0)
        .show(&ctx, |ui| {
            ui.add(egui::Slider::new(&mut model.settings.width, 80.0..=700.0).text("Width"));
            ui.add(egui::Slider::new(&mut model.settings.depth, 80.0..=700.0).text("Depth"));
            ui.add(egui::Slider::new(&mut model.settings.amplitude, 0.0..=140.0).text("Amplitude"));
            ui.add(
                egui::Slider::new(&mut model.settings.base_height, -80.0..=80.0)
                    .text("Base height"),
            );
            ui.add(
                egui::Slider::new(&mut model.settings.floor_depth, 0.0..=120.0).text("Floor depth"),
            );
            ui.separator();
            ui.add(egui::Slider::new(&mut model.settings.seed, 0..=10_000).text("Seed"));
            ui.add(
                egui::Slider::new(&mut model.settings.noise_scale, 0.002..=0.08)
                    .logarithmic(true)
                    .text("Scale"),
            );
            ui.add(egui::Slider::new(&mut model.settings.octaves, 1..=8).text("Octaves"));
            ui.add(
                egui::Slider::new(&mut model.settings.lacunarity, 1.01..=4.0).text("Lacunarity"),
            );
            ui.add(egui::Slider::new(&mut model.settings.gain, 0.0..=1.0).text("Gain"));
            ui.add(egui::Slider::new(&mut model.settings.ridge, 0.0..=1.0).text("Ridge"));
            ui.add(egui::Slider::new(&mut model.settings.offset_x, -80.0..=80.0).text("Offset X"));
            ui.add(egui::Slider::new(&mut model.settings.offset_z, -80.0..=80.0).text("Offset Z"));
            ui.separator();
            ui.add(
                egui::Slider::new(&mut model.settings.camera_distance, 240.0..=1100.0)
                    .text("Camera distance"),
            );
            ui.add(
                egui::Slider::new(&mut model.settings.camera_height, 40.0..=560.0)
                    .text("Camera height"),
            );
            ui.add(egui::Slider::new(&mut model.settings.camera_yaw, -PI..=PI).text("Yaw"));
            ui.add(egui::Slider::new(&mut model.settings.fov_degrees, 25.0..=70.0).text("FOV"));
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
    let field = app.sdf();
    let params = terrain_params(settings);
    let bounds = terrain_scene_bounds(params);

    field
        .configure()
        .bounds(bounds)
        .voxel_size(2.0)
        .brick_size(8)
        .narrow_band(10.0)
        .atlas_capacity(8192)
        .update_budget(SdfUpdateBudget::Unlimited);

    field.scene(|scene| {
        scene.union(sdf::terrain().key("terrain").params(params));
    });
}

fn terrain_params(settings: &Settings) -> SdfTerrainParams {
    SdfTerrainParams {
        size: Vec2::new(settings.width, settings.depth),
        amplitude: settings.amplitude,
        base_height: settings.base_height,
        floor_depth: settings.floor_depth,
        seed: settings.seed,
        noise_scale: settings.noise_scale,
        octaves: settings.octaves,
        lacunarity: settings.lacunarity,
        gain: settings.gain,
        ridge: settings.ridge,
        noise_offset: Vec2::new(settings.offset_x, settings.offset_z),
    }
    .clamped()
}

fn terrain_scene_bounds(params: SdfTerrainParams) -> SdfBounds {
    let half = params.size * 0.5;
    let top = params.base_height + params.amplitude;
    let bottom = params.base_height - params.amplitude - params.floor_depth;
    SdfBounds::from_min_max(
        Vec3::new(-half.x - 32.0, bottom - 24.0, -half.y - 32.0),
        Vec3::new(half.x + 32.0, top + 80.0, half.y + 32.0),
    )
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    let field = app.sdf();
    let s = &model.settings;
    let position = Vec3::new(
        s.camera_yaw.sin() * s.camera_distance,
        s.camera_height,
        s.camera_yaw.cos() * s.camera_distance,
    );

    draw.background().color(Color::srgb(0.025, 0.03, 0.035));
    draw.sdf(&field)
        .look_at(position, Vec3::new(0.0, s.base_height, 0.0))
        .fov_degrees(s.fov_degrees)
        .ambient(0.32)
        .diffuse(0.95)
        .light_dir(Vec3::new(-0.4, -0.9, -0.35))
        .debug(s.debug.sdf())
        .hit_epsilon(0.75)
        .normal_epsilon(2.0)
        .max_distance(1600.0);
}
