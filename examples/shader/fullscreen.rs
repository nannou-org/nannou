use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .simple_window(view)
        // Register our custom shader model to make it available for use in our drawing
        .shader_model::<ShaderModel>()
        .run()
}

struct Model {}

// This struct defines the data that will be passed to your shader
#[shader_model(fragment = "shaders/fullscreen.wgsl")]
struct ShaderModel {
    #[uniform(0)]
    mouse: Vec2,
}

fn model(app: &App) -> Model {
    Model {}
}

fn view(app: &App, _model: &Model, window: Entity) {
    // Begin drawing and configure the draw instance to use this frame's shader model.
    let draw = app.draw().shader_model(ShaderModel {
        mouse: app.mouse(),
    });

    // Draw a rectangle the size of the window shaded by our shader.
    let rect = app.window(window).rect();
    draw.rect().w_h(rect.w(), rect.h());
}
