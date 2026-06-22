use nannou::{prelude::*, sdf};

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
    let field = app.sdf();
    let t = smoothstep(0.0, 1.0, app.time().sin() * 0.5 + 0.5);

    field
        .configure()
        .quality(SdfQuality::High)
        .bounds(SdfBounds::from_min_max(
            pt3(-180.0, -180.0, -180.0),
            pt3(180.0, 180.0, 180.0),
        ));

    field.scene(|scene| {
        scene.union(sdf::sphere().key("planet").radius(80.0).color(STEEL_BLUE));
        scene.smooth_subtract(
            8.0,
            sdf::capsule()
                .key("tunnel")
                .from_to(pt3(-130.0, 0.0, 0.0), pt3(130.0, 0.0, 0.0))
                .radius(14.0 + 10.0 * t),
        );
        scene.smooth_union(
            12.0,
            sdf::torus()
                .key("ring")
                .major_radius(95.0)
                .minor_radius(8.0)
                .rotate_x(app.time() * 0.25)
                .color(ORANGE_RED),
        );
    });
}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    let field = app.sdf();

    draw.background().color(BLACK);
    draw.sdf(&field)
        .look_at(pt3(0.0, 70.0, 420.0), pt3(0.0, 0.0, 0.0));
}
