use nannou::prelude::*;

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

fn main() {
    nannou::app(model).model_ui().run();
}

#[derive(Resource, Reflect)]
struct Model {
    window: Entity,
    radius: f32,
    color: Hsva,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let window = app
        .new_window()
        .title("Nannou")
        .size(WIDTH as u32, HEIGHT as u32)
        .view(view) // The function that will be called for presenting graphics to a frame.
        .build();

    Model {
        window,
        radius: 40.0,
        color: Color::hsv(10.0, 0.5, 1.0).into(),
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();

    draw.background().color(BLACK);

    draw.ellipse()
        .x_y(100.0, 100.0)
        .radius(model.radius)
        .color(model.color);
}
