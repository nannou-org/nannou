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

    // Create a writer
    let spec = get_spec(&audio_host);
    let writer = hound::WavWriter::create("recorded.wav", spec).unwrap();
    let capture_model = CaptureModel { writer };

    // Kick off the audio thread
    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let audio_thread = std::thread::spawn(move || {
        let stream = audio_host
            .new_input_stream(capture_model)
            .capture(capture_fn)
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

    if !model.is_paused && app.elapsed_frames() % 30 < 20 {
        let draw = app.draw();
        draw.ellipse().w_h(100.0, 100.0).color(RED);
    }
}

// Get specification from default device format.
fn get_spec(audio_host: &nannou_audio::Host) -> hound::WavSpec {
    let default_input_config = audio_host
        .default_input_device()
        .unwrap()
        .default_input_config()
        .unwrap();

    let (bits_per_sample, sample_format) = match default_input_config.sample_format() {
        audio::cpal::SampleFormat::I16 => (16, hound::SampleFormat::Int),
        audio::cpal::SampleFormat::U16 => (16, hound::SampleFormat::Int),
        audio::cpal::SampleFormat::F32 => (32, hound::SampleFormat::Float),
    };

    hound::WavSpec {
        channels: default_input_config.channels(),
        sample_rate: default_input_config.sample_rate().0,
        bits_per_sample,
        sample_format,
    }
}
