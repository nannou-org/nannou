//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

fn main() {
    nannou::app(model).run();
}

struct Model {
    stream: audio::Stream<CaptureModel>,
}

struct CaptureModel {
    frames: Vec<f32>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    let audio_host = audio::Host::new();

    let model = CaptureModel {
        frames: vec![],
    };

    let stream = audio_host
        .new_input_stream(model)
        .capture(capture_fn)
        .build()
        .unwrap();

    Model { stream }
}

fn capture_fn(audio: &mut CaptureModel, buffer: &Buffer) {
    for frame in buffer.frames() {
        audio.frames.push(frame[0]);
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => {

        },
        Key::P => {

        },
        _ => {}
    }
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
