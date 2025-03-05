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
#[shader_model(fragment = "draw_custom_shader_model.wgsl")]
struct ShaderModel {
    #[uniform(0)]
    color: LinearRgba,
}

fn model(_app: &App) -> Model {
    Model {}
}

fn view(app: &App, _model: &Model, _window: Entity) {
    // Begin drawing
    let draw = app
        .draw()
        // Initialize our draw instance with our custom shader model
        .shader_model(ShaderModel { color: RED.into() });

    draw.ellipse().x(-200.0);

    // We can also map the shader model manually
    draw.ellipse()
        .map_shader_model(|mut mat| {
            mat.color = BLUE.into();
            mat
        })
        .x(200.0);
}
