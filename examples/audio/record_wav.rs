//! Records a WAV file using the default input device and default input format.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use std::fs::File;
use std::io::BufWriter;
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
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Create a writer
    let spec = get_spec(&audio_host);
    let writer = hound::WavWriter::create("recorded.wav", spec).unwrap();
    let capture_model = CaptureModel { writer };

    let stream = audio_host
        .new_input_stream(capture_model)
        .capture(capture_fn)
        .build()
        .unwrap();

    Model { stream }
}

// A function that captures the audio from the buffer and
// writes it into the the WavWriter.
fn capture_fn(audio: &mut CaptureModel, buffer: &Buffer) {
    // When the program ends, writer is dropped and data gets written to the disk.
    for frame in buffer.frames() {
        for sample in frame {
            audio
                .writer
                .write_sample(*sample)
                .expect("error while writing the sample");
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.stream.is_paused() {
                model.stream.play().unwrap();
            } else if model.stream.is_playing() {
                model.stream.pause().unwrap();
            }
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);

    if model.stream.is_playing() && app.elapsed_frames() % 30 < 20 {
        let draw = app.draw();
        draw.ellipse().w_h(100.0, 100.0).color(RED);

        draw.to_frame(app, &frame).unwrap();
    }
}

// Get specification from default device format.
fn get_spec(audio_host: &nannou_audio::Host) -> hound::WavSpec {
    let default_input_format = audio_host
        .default_input_device()
        .unwrap()
        .default_input_format()
        .unwrap();

    let (bits_per_sample, sample_format) = match default_input_format.data_type {
        audio::cpal::SampleFormat::I16 => (16, hound::SampleFormat::Int),
        audio::cpal::SampleFormat::U16 => (16, hound::SampleFormat::Int),
        audio::cpal::SampleFormat::F32 => (32, hound::SampleFormat::Float),
    };

    hound::WavSpec {
        channels: default_input_format.channels,
        sample_rate: default_input_format.sample_rate.0,
        bits_per_sample,
        sample_format,
    }
}
