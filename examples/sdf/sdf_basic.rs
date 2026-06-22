use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: Entity,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().size(900, 700).view(view).build();
    Model { _window }
}

fn update(app: &App, _model: &mut Model) {
    let sdf = app.sdf();
    let t = smoothstep(0.0, 1.0, app.time().sin() * 0.5 + 0.5);

    sdf.configure()
        .bounds(SdfBounds::from_min_max(
            pt3(-180.0, -180.0, -180.0),
            pt3(180.0, 180.0, 180.0),
        ))
        .voxel_size(1.0)
        .brick_size(8);

    sdf.transaction(|s| {
        s.sphere().key("planet").radius(80.0).color(STEEL_BLUE);

        s.smooth_subtract(8.0, |s| {
            s.capsule()
                .key("tunnel")
                .from_to(pt3(-130.0, 0.0, 0.0), pt3(130.0, 0.0, 0.0))
                .radius(14.0 + 10.0 * t);
        });

        s.smooth_union(12.0, |s| {
            s.torus()
                .key("ring")
                .major_radius(95.0)
                .minor_radius(8.0)
                .pitch(app.time() * 0.25)
                .color(ORANGE_RED);
        });
    });
}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    let sdf = app.sdf();

    draw.background().color(BLACK);
    draw.sdf(&sdf).camera(SdfCamera {
        position: pt3(0.0, 70.0, 420.0),
        target: pt3(0.0, 0.0, 0.0),
        ..Default::default()
    });
}
