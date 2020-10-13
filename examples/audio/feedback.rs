use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use ringbuf::{Consumer, Producer, RingBuffer};

fn main() {
    nannou::app(model).run();
}

struct Model {
    in_stream: audio::Stream<InModel>,
    out_stream: audio::Stream<OutModel>,
}

struct InModel {
    pub sender: Producer<[f32; 2]>,
}

struct OutModel {
    pub receiver: Consumer<[f32; 2]>,
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

    let ring_buffer = RingBuffer::<[f32; 2]>::new(1024 * 2); // Add some latency
    let (mut prod, cons) = ring_buffer.split();
    let in_model = InModel { sender: prod };
    let in_stream = audio_host
        .new_input_stream(in_model)
        .capture(pass_in)
        .build()
        .unwrap();

    let out_model = OutModel { receiver: cons };
    let out_stream = audio_host
        .new_output_stream(out_model)
        .render(pass_out)
        .build()
        .unwrap();

    Model {
        in_stream,
        out_stream,
    }
}

fn pass_in(model: &mut InModel, buffer: &Buffer) {
    for frame in buffer.frames() {
        model.sender.push([frame[0], frame[1]]).unwrap();
    }
}

fn pass_out(model: &mut OutModel, buffer: &mut Buffer) {
    for frame in buffer.frames_mut() {
        let recording_frame = match model.receiver.pop() {
            Some(f) => f,
            None => [0.0, 0.0],
        };
        for (sample, recording_sample) in frame.iter_mut().zip(&recording_frame) {
            *sample = *recording_sample * 0.5;
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            model.in_stream.play().unwrap();
            model.out_stream.play().unwrap();
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);
}
