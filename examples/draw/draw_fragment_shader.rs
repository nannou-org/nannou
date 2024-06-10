use nannou::prelude::*;

struct Model {}

fn main() {
    nannou::app(model)
        .simple_window(view)
        // We need to initialize the fragment shader with the path to the shader file
        // in our assets directory so that it will be available to use as a material.
        .init_fragment_shader::<"draw_fragment_shader.wgsl">()
        .run()
}

fn model(app: &App) -> Model {
    Model {}
}

fn view(app: &App, model: &Model, window: Entity) {
    // Begin drawing
    let draw = app.draw();
    // Draw a full-screen quad
    let window = app.main_window();
    let window_rect = window.rect();
    draw.rect()
        // Specify the shader to use for this draw call
        .fragment_shader::<"draw_fragment_shader.wgsl">()
        .w_h(window_rect.w(), window_rect.h())
        .color(WHITE);
}
