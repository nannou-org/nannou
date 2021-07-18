//! Feeds back the input stream directly into the output stream
//!
//! You can play and pause the streams by pressing space key
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use ringbuf::{Consumer, Producer, RingBuffer};

fn main() {
    nannou::app(model).run();
}

struct Model {
    in_stream: audio::Stream<InputModel>,
    out_stream: audio::Stream<OutputModel>,
}

struct InputModel {
    pub producer: Producer<f32>,
}

struct OutputModel {
    pub consumer: Consumer<f32>,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .windowsos_drag_and_drop(false) // Required for Windows (no effect otherwise)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

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

    Model {
        in_stream,
        out_stream,
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
            let recorded_sample = match model.consumer.pop() {
                Some(f) => f,
                None => 0.0,
            };
            *sample = recorded_sample;
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.in_stream.is_paused() {
                model.in_stream.play().unwrap();
                model.out_stream.play().unwrap();
            } else {
                model.in_stream.pause().unwrap();
                model.out_stream.pause().unwrap();
            }
        }
        _ => {}
    }
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
