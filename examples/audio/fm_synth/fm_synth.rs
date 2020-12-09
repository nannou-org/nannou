use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

use dasp::signal::Signal;
use musical_keyboard as mkb;

mod adsr;
mod biquad;
mod gui;
mod synth;

fn main() {
    nannou::app(model).update(update).run();
}

pub struct Parameters {
    master_frequency: f32,
    op1: synth::Operator,
    op2: synth::Operator,
    filter: synth::Filter,
    note_on_off: bool,
    master_volume: f32,
}

struct Model {
    stream: audio::Stream<Audio>,
    ui: Ui,
    ids: gui::Ids,
    parameters: Parameters,
    synth_control: synth::Synth,
    musical_keyboard: mkb::MusicalKeyboard,
}

struct Audio {
    master_volume: f32,
    fm_synth: Box<dyn Signal<Frame = f64> + Send>,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .size(240, 890)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .view(view)
        .build()
        .unwrap();

    let op1 = synth::Operator {
        pitch: synth::Pitch {
            ratio: 3.5,
            ratio_offset: 0.0,
        },
        env: synth::Envelope {
            attack: 0.71,
            decay: 0.3,
            sustain: 1.0,
            release: 0.8,
        },
        amp: 800.0,
    };

    let op2 = synth::Operator {
        pitch: synth::Pitch {
            ratio: 0.25,
            ratio_offset: 0.0,
        },
        env: synth::Envelope {
            attack: 0.1,
            decay: 0.3,
            sustain: 1.0,
            release: 0.8,
        },
        amp: 0.5,
    };

    let filter = synth::Filter {
        cutoff: 1000.0,
        resonance: 0.707,
        filter_type: 0,
        peak_gain: 0.0,
    };

    let master_frequency = 100.0;
    let master_volume = 0.8;
    let sample_rate = 44100.0;
    let (synth_control, synth_signal) =
        synth::Synth::new(sample_rate, master_frequency, &op1, &op2, &filter);

    let audio_model = Audio {
        master_volume,
        fm_synth: synth_signal,
    };

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .build()
        .unwrap();

    let parameters = Parameters {
        master_frequency,
        op1,
        op2,
        filter,
        note_on_off: false,
        master_volume,
    };

    let musical_keyboard = mkb::MusicalKeyboard::default();

    // Create the UI.
    let mut ui = app.new_ui().build().unwrap();

    // Generate some ids for our widgets.
    let ids = gui::Ids::new(ui.widget_id_generator());

    Model {
        stream,
        ui,
        ids,
        parameters,
        synth_control,
        musical_keyboard,
    }
}

// A function that renders the given `Audio` to the given `Buffer`.
fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    for frame in buffer.frames_mut() {
        for channel in frame {
            *channel = audio.fm_synth.next() as f32 * audio.master_volume;
        }
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Calling `set_widgets` allows us to instantiate some widgets.
    let ui = model.ui.set_widgets();

    gui::update(
        ui,
        &mut model.ids,
        &mut model.parameters,
        &mut model.synth_control.producers,
    );

    let volume = model.parameters.master_volume.clone();
    model
        .stream
        .send(move |audio| {
            audio.master_volume = volume;
        })
        .unwrap();
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    if let Some(k) = convert_key(key) {
        if let Some(note) = model.musical_keyboard.key_pressed(k) {
            let nn = convert_key_note_number(note.letter, note.octave);
            model.parameters.master_frequency = note_to_frequency(nn);

            if let Ok(_) =
                model
                    .synth_control
                    .producers
                    .mod_hz
                    .push(crate::calculate_operator_frequency(
                        model.parameters.master_frequency,
                        &model.parameters.op1,
                    ) as f64)
            {}

            if let Ok(_) =
                model
                    .synth_control
                    .producers
                    .carrier_hz
                    .push(crate::calculate_operator_frequency(
                        model.parameters.master_frequency,
                        &model.parameters.op2,
                    ) as f64)
            {}

            if !model.parameters.note_on_off {
                if model
                    .synth_control
                    .producers
                    .mod_env_on_off
                    .push(true)
                    .is_ok()
                    && model
                        .synth_control
                        .producers
                        .carrier_env_on_off
                        .push(true)
                        .is_ok()
                {
                    model.parameters.note_on_off = true;
                }
            }
        }
    }
}

fn key_released(_app: &App, model: &mut Model, key: Key) {
    if let Some(k) = convert_key(key) {
        let _off = model.musical_keyboard.key_released(k);
        if model
            .synth_control
            .producers
            .mod_env_on_off
            .push(false)
            .is_ok()
            && model
                .synth_control
                .producers
                .carrier_env_on_off
                .push(false)
                .is_ok()
        {
            model.parameters.note_on_off = false;
        }
    };
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(rgb(0.1, 0.1, 0.1));

    // Draw the state of the `Ui` to the frame.
    model.ui.draw_to_frame(app, &frame).unwrap();
}

pub fn calculate_operator_frequency(master_frequency: f32, op: &synth::Operator) -> f32 {
    master_frequency * (op.pitch.ratio + op.pitch.ratio_offset)
}

pub fn note_to_frequency(n: i32) -> f32 {
    const BASE_A4: f32 = 440.0; // A4 = 440Hz
    BASE_A4 * 2.0.powf((n as f32 - 49.0) / 12.0)
}

pub fn convert_key_note_number(key: mkb::Letter, octave: i32) -> i32 {
    let n = match key {
        mkb::Letter::C => 1,
        mkb::Letter::Csh => 2,
        mkb::Letter::Db => 2,
        mkb::Letter::D => 3,
        mkb::Letter::Dsh => 4,
        mkb::Letter::Eb => 4,
        mkb::Letter::E => 5,
        mkb::Letter::F => 6,
        mkb::Letter::Fsh => 7,
        mkb::Letter::Gb => 7,
        mkb::Letter::G => 8,
        mkb::Letter::Gsh => 9,
        mkb::Letter::Ab => 9,
        mkb::Letter::A => 10,
        mkb::Letter::Ash => 11,
        mkb::Letter::Bb => 11,
        mkb::Letter::B => 12,
    };
    let offset = 3;
    (12 * octave) + n + offset
}

pub fn convert_key(key: Key) -> Option<mkb::Key> {
    let k = match key {
        Key::A => mkb::Key::A,
        Key::W => mkb::Key::W,
        Key::S => mkb::Key::S,
        Key::E => mkb::Key::E,
        Key::D => mkb::Key::D,
        Key::F => mkb::Key::F,
        Key::T => mkb::Key::T,
        Key::G => mkb::Key::G,
        Key::Y => mkb::Key::Y,
        Key::H => mkb::Key::H,
        Key::U => mkb::Key::U,
        Key::J => mkb::Key::J,
        Key::K => mkb::Key::K,
        Key::O => mkb::Key::O,
        Key::L => mkb::Key::L,
        Key::P => mkb::Key::P,
        Key::Semicolon => mkb::Key::Semicolon,
        Key::Apostrophe => mkb::Key::Quote,
        Key::Z => mkb::Key::Z,
        Key::X => mkb::Key::X,
        Key::C => mkb::Key::C,
        Key::V => mkb::Key::V,
        _ => return None,
    };
    Some(k)
}
