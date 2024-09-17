use nannou::noise::NoiseFn;
use nannou::prelude::*;

fn main() {
    nannou::app(model)
        // Register our custom material to make it available for use in our drawing
        .shader_model::<VideoShaderModel>()
        .run();
}

#[derive(Reflect)]
struct Model {
    window: Entity,
    camera: Entity,
    video: Handle<Video>,
}

// This struct defines the data that will be passed to your shader
#[shader_model(fragment = "draw_video_material.wgsl")]
struct VideoShaderModel {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
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

    let draw = app
        .draw()
        // Initialize our draw instance with our custom material
        .shader_model(VideoShaderModel {
            texture: video.texture.clone(),
        });

    draw.rect().w_h(640.0, 400.0);
}
