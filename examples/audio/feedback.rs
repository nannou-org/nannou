//! Feeds back the input stream directly into the output stream
//!
//! You can play and pause the streams by pressing space key
use ringbuf::{Consumer, Producer, RingBuffer};
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

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

struct InputModel {
    pub producer: Producer<f32>,
}

struct OutputModel {
    pub consumer: Consumer<f32>,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window().key_pressed(key_pressed).view(view).build();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Create a ring buffer and split it into producer and consumer
    let latency_samples = 1024;
    let ring_buffer = RingBuffer::<f32>::new(latency_samples * 2); // Add some latency
    let (mut prod, cons) = ring_buffer.split();
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        prod.push(0.0).unwrap();
    }

    // Kick off the audio thread
    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let audio_thread = std::thread::spawn(move || {
        // Create input model and input stream using that model
        let in_model = InputModel { producer: prod };
        let in_stream = audio_host
            .new_input_stream(in_model)
            .capture(pass_in)
            .build()
            .unwrap();

        // Create output model and output stream using that model
        let out_model = OutputModel { consumer: cons };
        let out_stream = audio_host
            .new_output_stream(out_model)
            .render(pass_out)
            .build()
            .unwrap();

        in_stream.play().unwrap();
        out_stream.play().unwrap();

        let in_stream = in_stream;
        let out_stream = out_stream;
        loop {
            match audio_rx.recv() {
                Ok(AudioCommand::Play) => {
                    in_stream.play().unwrap();
                    out_stream.play().unwrap();
                }
                Ok(AudioCommand::Pause) => {
                    in_stream.pause().unwrap();
                    out_stream.pause().unwrap();
                }
                Ok(AudioCommand::Exit) => {
                    in_stream.pause().ok();
                    out_stream.pause().ok();
                    break;
                }
                Err(_) => break,
            }
        }
    });

    Model {
        audio_thread,
        audio_tx,
        is_paused: false,
    }
}

fn pass_in(model: &mut InputModel, buffer: &Buffer) {
    for frame in buffer.frames() {
        for sample in frame {
            model.producer.push(*sample).ok();
        }
    }
}

fn pass_out(model: &mut OutputModel, buffer: &mut Buffer) {
    for frame in buffer.frames_mut() {
        for sample in frame {
            let recorded_sample = model.consumer.pop().unwrap_or(0.0);
            *sample = recorded_sample;
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

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(DIM_GRAY);
}
