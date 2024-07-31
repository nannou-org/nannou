use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

fn main() {
    nannou::app(model).exit(exit).run();
}

struct Model {
    audio_thread: std::thread::JoinHandle<()>,
    audio_tx: std::sync::mpsc::Sender<AudioCommand>,
    is_paused: bool,
}

pub enum AudioCommand {
    PlaySound(audrey::read::BufFileReader),
    Exit,
}

struct Audio {
    sounds: Vec<audrey::read::BufFileReader>,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window().key_pressed(key_pressed).view(view).build();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Initialise the state that we want to live on the audio thread.
    let sounds = vec![];
    let model = Audio { sounds };

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
                Ok(AudioCommand::PlaySound(sound)) => {
                    stream
                        .send(move |audio| {
                            audio.sounds.push(sound);
                        })
                        .unwrap();
                }
                Ok(AudioCommand::Exit) => {
                    stream.pause().ok();
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

// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play the audio file.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let mut have_ended = vec![];
    let len_frames = buffer.len_frames();

    // Sum all of the sounds onto the buffer.
    for (i, sound) in audio.sounds.iter_mut().enumerate() {
        let mut frame_count = 0;
        let file_frames = sound.frames::<[f32; 2]>().filter_map(Result::ok);
        for (frame, file_frame) in buffer.frames_mut().zip(file_frames) {
            for (sample, file_sample) in frame.iter_mut().zip(&file_frame) {
                *sample += *file_sample;
            }
            frame_count += 1;
        }

        // If the sound yielded less samples than are in the buffer, it must have ended.
        if frame_count < len_frames {
            have_ended.push(i);
        }
    }

    // Remove all sounds that have ended.
    for i in have_ended.into_iter().rev() {
        audio.sounds.remove(i);
    }
}

fn key_pressed(app: &App, model: &mut Model, key: KeyCode) {
    if key == KeyCode::Space {
        let assets = app.assets_path();
        let path = assets.join("sounds").join("thumbpiano.wav");
        let sound = audrey::open(path).expect("failed to load sound");
        model.audio_tx.send(AudioCommand::PlaySound(sound)).ok();
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
