use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
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
    let isf = model.isf.clone();
    let texture = model.texture_1.clone();
    app.command_scope(move |mut commands| {
        commands.queue(move |world: &mut World| {
            // Wait until the ISF asset has loaded before setting its inputs.
            if world
                .resource::<bevy_asset::Assets<Isf>>()
                .get(&isf)
                .is_none()
            {
                return;
            }
            world.resource_mut::<IsfInputs>().insert(
                "inputImage".to_string(),
                IsfInputValue::Image(texture.clone()),
            );
        });
    });
}

fn view(app: &App, _model: &Model) {
    let _draw = app.draw();
}
