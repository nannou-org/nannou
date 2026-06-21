//! Records a WAV file using the default input device and default input format.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".
extern crate hound;

use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

type WavWriter = hound::WavWriter<BufWriter<File>>;

fn main() {
    nannou::app(model).exit(exit).run();
}

pub enum AudioCommand {
    Play,
    Pause,
    Exit,
}

struct Model {
    audio_thread: JoinHandle<()>,
    audio_tx: Sender<AudioCommand>,
    is_paused: bool,
}

struct CaptureModel {
    writer: WavWriter,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window().key_pressed(key_pressed).view(view).build();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Record using the default input device's default config. We pin the stream to this device
    // and config so the captured data matches the WAV header exactly (nannou would otherwise
    // negotiate its own default sample rate and channel count).
    let device = audio_host
        .default_input_device()
        .expect("no default input device available");
    let config = device
        .default_input_config()
        .expect("failed to read the default input config");
    let channels = config.channels();
    let sample_rate = config.sample_rate();

    // The stream captures samples as `f32` regardless of the device's native format, so the WAV
    // is always written as 32-bit float.
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let writer = hound::WavWriter::create("recorded.wav", spec).unwrap();
    let capture_model = CaptureModel { writer };

    // Kick off the audio thread
    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let audio_thread = std::thread::spawn(move || {
        let stream = audio_host
            .new_input_stream(capture_model)
            .capture(capture_fn)
            .device(device)
            .channels(channels as usize)
            .sample_rate(sample_rate)
            .build()
            .unwrap();

        stream.play().unwrap();

        loop {
            match audio_rx.recv() {
                Ok(AudioCommand::Play) => {
                    stream.play().unwrap();
                }
                Ok(AudioCommand::Pause) => {
                    stream.pause().unwrap();
                }
                Ok(AudioCommand::Exit) => {
                    stream.pause().ok();
                    break;
                }
                _ => (),
            }
        }
    });

    Model {
        audio_thread,
        audio_tx,
        is_paused: false,
    }
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

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    if key == KeyCode::Space {
        if model.is_paused {
            model.audio_tx.send(AudioCommand::Play).ok();
            model.is_paused = false;
        } else {
            model.audio_tx.send(AudioCommand::Pause).ok();
            model.is_paused = true;
        }
    }
}

fn exit(_app: &App, model: Model) {
    model.audio_tx.send(AudioCommand::Exit).ok();
    model.audio_thread.join().ok();
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(DIM_GRAY);

    if !model.is_paused {
        let draw = app.draw();
        draw.ellipse().w_h(100.0, 100.0).color(RED);
    }
}
