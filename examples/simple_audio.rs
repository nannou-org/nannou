extern crate nannou;

use nannou::audio::{self, Buffer};
use nannou::prelude::*;
use std::f64::consts::PI;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    stream: audio::Stream<Audio>,
}

struct Audio {
    phase: f64,
    hz: f64,
}

// A function that renders the given `Audio` to the given `Buffer`, returning the result of both.
//
// In this case we play a simple sine wave at the audio's current frequency in `hz`..
fn audio(mut audio: Audio, mut buffer: Buffer) -> (Audio, Buffer) {
    let sample_rate = buffer.sample_rate() as f64;
    let volume = 0.5;
    for frame in buffer.frames_mut() {
        let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        audio.phase += audio.hz / sample_rate;
        audio.phase %= sample_rate;
        for channel in frame {
            *channel = sine_amp * volume;
        }
    }
    (audio, buffer)
}

fn model(app: &App) -> Model {
    // Initialise the state that we want to live on the audio thread.
    let model = Audio {
        phase: 0.0,
        hz: 440.0,
    };
    let stream = app.audio.new_output_stream(model, audio).build().unwrap();
    Model { stream }
}

fn event(_app: &App, model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => match event {
            KeyPressed(key) => match key {
                // Pause or unpause the audio when Space is pressed.
                Key::Space => {
                    if model.stream.is_playing() {
                        model.stream.pause();
                    } else {
                        model.stream.play();
                    }
                }

                // Raise the frequency when the up key is pressed.
                Key::Up => {
                    model
                        .stream
                        .send(|audio| {
                            audio.hz += 10.0;
                        })
                        .unwrap();
                }

                // Lower the frequency when the down key is pressed.
                Key::Down => {
                    model
                        .stream
                        .send(|audio| {
                            audio.hz -= 10.0;
                        })
                        .unwrap();
                }

                _ => (),
            },

            _ => (),
        },
        Event::Update(_update) => {}
        _ => (),
    }
    model
}

fn view(_app: &App, _model: &Model, frame: Frame) -> Frame {
    frame.clear_all(DARK_CHARCOAL);
    frame
}
