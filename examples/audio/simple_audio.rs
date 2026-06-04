use std::f64::consts::PI;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

fn main() {
    nannou::app(model).exit(exit).run();
}

enum AudioCommand {
    Play,
    Pause,
    IncreaseFrequency(f64),
    DecreaseFrequency(f64),
    Exit,
}

struct Model {
    audio_thread: JoinHandle<()>,
    audio_tx: Sender<AudioCommand>,
    is_paused: bool,
}

struct Audio {
    phase: f64,
    hz: f64,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window().key_pressed(key_pressed).view(view).build();

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Initialise the state that we want to live on the audio thread.
    let model = Audio {
        phase: 0.0,
        hz: 440.0,
    };

    // Kick off the audio thread.
    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let audio_thread = std::thread::spawn(move || {
        let stream = audio_host
            .new_output_stream(model)
            .render(audio)
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
                Ok(AudioCommand::IncreaseFrequency(hz)) => {
                    stream
                        .send(move |audio| {
                            audio.hz += hz;
                        })
                        .unwrap();
                }
                Ok(AudioCommand::DecreaseFrequency(hz)) => {
                    stream
                        .send(move |audio| {
                            audio.hz -= hz;
                        })
                        .unwrap();
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

// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play a simple sine wave at the audio's current frequency in `hz`.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
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
}

fn key_pressed(_app: &App, model: &mut Model, key: KeyCode) {
    match key {
        // Pause or unpause the audio when Space is pressed.
        KeyCode::Space => {
            if model.is_paused {
                model.audio_tx.send(AudioCommand::Play).ok();
                model.is_paused = false;
            } else {
                model.audio_tx.send(AudioCommand::Pause).ok();
                model.is_paused = true;
            }
        }
        // Raise the frequency when the up key is pressed.
        KeyCode::ArrowUp => {
            model
                .audio_tx
                .send(AudioCommand::IncreaseFrequency(10.0))
                .ok();
        }
        // Lower the frequency when the down key is pressed.
        KeyCode::ArrowDown => {
            model
                .audio_tx
                .send(AudioCommand::DecreaseFrequency(10.0))
                .ok();
        }
        _ => {}
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
