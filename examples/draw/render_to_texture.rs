//! Demonstrates multiple nannou render-to-texture targets.
//!
//! Each `RenderTexture` owns an independent image and camera, while both the offscreen
//! passes and the final presentation use nannou's `Draw` API.

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    red_target: RenderTexture,
    blue_target: RenderTexture,
}

fn model(app: &App) -> Model {
    app.new_window().size(800, 500).primary().view(view).build();

    Model {
        red_target: app.new_render_texture(256, 256),
        blue_target: app.new_render_texture(256, 256),
    }
}

fn update(app: &App, model: &mut Model) {
    if let Some(draw) = model.red_target.draw(app) {
        draw.background().color(WHITE);
        draw.rect()
            .w_h(160.0, 160.0)
            .color(RED)
            .stroke(BLACK)
            .stroke_weight(6.0)
            .rotate(app.time());
    }

    if let Some(draw) = model.blue_target.draw(app) {
        draw.background().color(WHITE);
        draw.ellipse()
            .w_h(170.0, 120.0)
            .color(BLUE)
            .stroke(BLACK)
            .stroke_weight(6.0)
            .rotate(-app.time());
    }
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(DARK_GRAY);

    draw.rect()
        .x_y(-190.0, 0.0)
        .w_h(256.0, 256.0)
        .texture(model.red_target.image());
    draw.rect()
        .x_y(190.0, 0.0)
        .w_h(256.0, 256.0)
        .texture(model.blue_target.image());
}
