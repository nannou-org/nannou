use nannou::prelude::*;

fn main() {
    nannou::app(model).model_ui().update(update).run();
}

#[derive(Reflect)]
struct Model {
    window: Entity,
    camera: Entity,
    isf: Handle<Isf>,
    texture_1: Handle<Image>,
    texture_2: Handle<Image>,
}

fn model(app: &App) -> Model {
    let camera = app.new_camera().build();
    let window = app
        .new_window()
        .camera(camera)
        .primary()
        .size_pixels(1024, 512)
        .view(view)
        .build();

    let isf = app.asset_server().load("isf/Test-MultiPassRendering.fs");
    let texture_1 = app.asset_server().load("images/nature/nature_1.jpg");
    let texture_2 = app.asset_server().load("images/nature/nature_2.jpg");
    Model {
        window,
        camera,
        isf,
        texture_1,
        texture_2,
    }
}

fn update(app: &App, model: &mut Model) {
    let Some(isf) = app.assets_mut::<Isf>().get(&model.isf) else {
        return;
    };

    let mut inputs = app.resource_mut::<IsfInputs>();
    inputs.insert("inputImage".to_string(), IsfInputValue::Image(model.texture_1.clone()));
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
}
