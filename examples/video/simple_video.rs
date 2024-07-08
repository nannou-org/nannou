use nannou::noise::NoiseFn;
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
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

    let video = app.asset_server().load_with_settings("video/file_example_MP4_640_3MG.mp4", |settings: &mut VideoLoaderSettings| {
        settings.options.insert("sws_flags".to_string(), "fast_bilinear".to_string());
    });
    Model {
        window,
        camera,
        video,
    }
}

fn update(app: &App, model: &mut Model) {

}

fn view(app: &App, model: &Model) {
    let Some(video) = app.assets().get(&model.video) else {
        return;
    };

    let draw = app.draw();
    draw.rect()
        .w_h(100.0, 100.0)
        .texture(&video.texture);
}
