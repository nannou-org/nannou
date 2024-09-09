use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .simple_window(view)
        // Register our custom material to make it available for use in our drawing
        .shader_model::<ShaderModel>()
        .run()
}

struct Model {}

// This struct defines the data that will be passed to your shader
#[shader_model(fragment = "draw_custom_material.wgsl")]
struct ShaderModel {
    #[uniform(0)]
    color: LinearRgba,
}

fn model(app: &App) -> Model {
    Model {}
}

fn view(app: &App, model: &Model, window: Entity) {
    // Begin drawing
    let draw = app
        .draw()
        // Initialize our draw instance with our custom material
        .material(ShaderModel { color: RED.into() });

    draw.ellipse().x(-200.0);

    // We can also map the material manually
    draw.ellipse()
        .map_material(|mut mat| {
            mat.color = BLUE.into();
            mat
        })
        .x(200.0);
}
