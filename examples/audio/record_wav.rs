//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
extern crate hound;

type WavWriter = hound::WavWriter<BufWriter<File>>;

fn main() {
    nannou::app(model).run();
}

struct Model {
    stream: audio::Stream<CaptureModel>,
}

struct CaptureModel {
    writer: WavWriter,
}

fn model(app: &App) -> Model {
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    let audio_host = audio::Host::new();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create("recorded.wav", spec).unwrap();
    let capture_model = CaptureModel { writer };

    let stream = audio_host
        .new_input_stream(capture_model)
        .capture(capture_fn)
        .build()
        .unwrap();

    Model { stream }
}

fn capture_fn(audio: &mut CaptureModel, buffer: &Buffer) {
    for frame in buffer.frames() {
        audio
            .writer
            .write_sample(frame[0] as f32)
            .expect("error while writing");
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => {}
        Key::P => {}
        _ => {}
    }
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
