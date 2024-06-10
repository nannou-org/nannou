use nannou::prelude::*;
use nannou::prelude::primitive::Primitive;

fn main() {
    nannou::app(model)
        .simple_window(view)
        // Register our custom material to make it available for use in our drawing
        .init_custom_material::<CustomMaterial>()
        .run()
}

struct Model {}

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
struct CustomMaterial {
    #[uniform(0)]
    color: LinearRgba,
}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "draw_custom_material.wgsl".into()
    }
}


fn model(app: &App) -> Model {
    Model {}
}

fn view(app: &App, model: &Model, window: Entity) {
    // Begin drawing
    let draw = app.draw()
        // Initialize our draw instance with our custom material
        .material(CustomMaterial {
            color: RED.into(),
        });

    draw.ellipse()
        .x(-200.0);

    // We can also map the material manually
    draw.ellipse()
        .map_material(|mut mat| {
            mat.color = BLUE.into();
            mat
        })
        .x(200.0);
}
