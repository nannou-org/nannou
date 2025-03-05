use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

#[derive(Reflect)]
struct Model {
    window: Entity,
    camera: Entity,
    video: Handle<Video>,
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

    let video = app
        .asset_server()
        .load("video/file_example_MP4_640_3MG.mp4");
    Model {
        window,
        camera,
        video,
    }
}
fn view(app: &App, model: &Model) {
    let assets = app.assets();
    let Some(video) = assets.get(&model.video) else {
        return;
    };

    let draw = app.draw();
    draw.rect().w_h(640.0, 400.0).texture(&video.texture);
}
