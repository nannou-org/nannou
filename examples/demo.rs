#[macro_use]
extern crate conrod_core;
extern crate nannou;
extern crate nannou_timeline as timeline;
extern crate pitch_calc;
extern crate time_calc;

use nannou::prelude::*;
use nannou::ui::prelude::*;
use pitch_calc as pitch;
use std::iter::once;
use time_calc as time;
use timeline::track::automation::{BangValue as Bang, Envelope, Point, ToggleValue as Toggle};
use timeline::track::piano_roll;
use timeline::{bars, track};

const BPM: time::calc::Bpm = 140.0;
const ONE_SECOND_MS: time::calc::Ms = 1_000.0;
const PPQN: time::Ppqn = 9600;
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    ui: Ui,
    ids: Ids,
    timeline_data: TimelineData,
}

struct TimelineData {
    playhead_secs: f64,
    bars: Vec<time::TimeSig>,
    notes: Vec<piano_roll::Note>,
    tempo_envelope: track::automation::numeric::Envelope<f32>,
    octave_envelope: track::automation::numeric::Envelope<i32>,
    toggle_envelope: track::automation::toggle::Envelope,
    bang_envelope: track::automation::bang::Envelope,
}

// Create all of our unique `WidgetId`s with the `widget_ids!` macro.
conrod_core::widget_ids! {
    struct Ids {
        window,
        ruler,
        timeline,
    }
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("Timeline Demo")
        .view(view)
        .build()
        .unwrap();

    // Create the UI.
    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids::new(ui.widget_id_generator());

    // Start the playhead at the beginning.
    let playhead_secs = 0.0;

    // A sequence of bars with varying time signatures.
    let bars = vec![
        time::TimeSig { top: 4, bottom: 4 },
        time::TimeSig { top: 4, bottom: 4 },
        time::TimeSig { top: 6, bottom: 8 },
        time::TimeSig { top: 6, bottom: 8 },
        time::TimeSig { top: 4, bottom: 4 },
        time::TimeSig { top: 4, bottom: 4 },
        time::TimeSig { top: 7, bottom: 8 },
        time::TimeSig { top: 7, bottom: 8 },
    ];

    let notes = bars::WithStarts::new(bars.iter().cloned(), PPQN)
        .enumerate()
        .map(|(i, (time_sig, start))| {
            let end = start + time_sig.ticks_per_bar(PPQN);
            let period = timeline::Period { start, end };
            let pitch = pitch::Step((24 + (i * 5) % 12) as f32).to_letter_octave();
            piano_roll::Note { period, pitch }
        })
        .collect();

    let tempo_envelope = {
        let start = Point {
            ticks: time::Ticks(0),
            value: 20.0,
        };
        let points = bars::Periods::new(bars.iter().cloned(), PPQN)
            .enumerate()
            .map(|(i, period)| Point {
                ticks: period.end,
                value: 20.0 + (i + 1) as f32 * 60.0 % 220.0,
            });
        Envelope::from_points(once(start).chain(points), 20.0, 240.0)
    };

    let octave_envelope = {
        let start = Point {
            ticks: time::Ticks(0),
            value: 0,
        };
        let points = bars::WithStarts::new(bars.iter().cloned(), PPQN)
            .enumerate()
            .flat_map(|(i, (ts, mut start))| {
                let bar_end = start + ts.ticks_per_bar(PPQN);
                let mut j = 0;
                std::iter::from_fn(move || {
                    if start >= bar_end {
                        return None;
                    }

                    let end = start + time::Ticks(PPQN as _);
                    let end = if end > bar_end { bar_end } else { end };
                    let point = Point {
                        ticks: end,
                        value: 1 + ((i as i32 + j as i32) * 3) % 12,
                    };
                    start = end;
                    j += 1;
                    Some(point)
                })
            });
        Envelope::from_points(once(start).chain(points), 0, 12)
    };

    let toggle_envelope = {
        let start = Point {
            ticks: time::Ticks(0),
            value: Toggle(random()),
        };
        let points = bars::Periods::new(bars.iter().cloned(), PPQN).map(|period| Point {
            ticks: period.end,
            value: Toggle(random()),
        });
        Envelope::from_points(once(start).chain(points), Toggle(false), Toggle(true))
    };

    let bang_envelope = {
        let points = bars::Periods::new(bars.iter().cloned(), PPQN).map(|period| Point {
            ticks: period.start,
            value: Bang,
        });
        Envelope::from_points(points, Bang, Bang)
    };

    let timeline_data = TimelineData {
        playhead_secs,
        bars,
        notes,
        tempo_envelope,
        octave_envelope,
        toggle_envelope,
        bang_envelope,
    };

    Model {
        _window,
        ui,
        ids,
        timeline_data,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let Model {
        ref ids,
        ref mut ui,
        ref mut timeline_data,
        ..
    } = *model;

    // Update the user interface.
    set_widgets(&mut ui.set_widgets(), ids, timeline_data);

    // Update the playhead.
    let total_duration_ticks =
        timeline::bars_duration_ticks(timeline_data.bars.iter().cloned(), PPQN);
    let total_duration_ms = total_duration_ticks.ms(BPM, PPQN);
    let total_duration_secs = total_duration_ms / ONE_SECOND_MS;
    let delta_secs = update.since_last.secs();
    timeline_data.playhead_secs = (timeline_data.playhead_secs + delta_secs) % total_duration_secs;
}

fn view(app: &App, model: &Model, frame: &Frame) {
    model.ui.draw_to_frame(app, frame).unwrap();
}

// Update / draw the Ui.
fn set_widgets(ui: &mut UiCell, ids: &Ids, data: &mut TimelineData) {
    use timeline::Timeline;

    // Main window canvas.
    widget::Canvas::new()
        .border(0.0)
        .color(ui::color::DARK_CHARCOAL.alpha(0.5))
        .set(ids.window, ui);

    let TimelineData {
        ref mut playhead_secs,
        ref bars,
        ref notes,
        ref mut tempo_envelope,
        ref mut octave_envelope,
        ref mut toggle_envelope,
        ref mut bang_envelope,
    } = *data;

    let ticks = time::Ms(*playhead_secs * ONE_SECOND_MS).to_ticks(BPM, PPQN);
    let color = ui::color::LIGHT_BLUE;

    ////////////////////
    ///// TIMELINE /////
    ////////////////////
    //
    // Set the `Timeline` widget.
    //
    // This returns a context on which we can begin setting our tracks, playhead and scrollbar.
    //
    // The context is used in three stages:
    //
    // 1. `PinnedTracks` for setting tracks that should be pinned to the top of the timeline.
    // 2. `Tracks` for setting regular tracks.
    // 3. `Final` for setting the `Playhead` and `Scrollbar` widgets after all tracks are set.

    let context = Timeline::new(bars.iter().cloned(), PPQN)
        .playhead(ticks)
        .color(color)
        .wh_of(ids.window)
        .middle_of(ids.window)
        .border(1.0)
        .border_color(ui::color::CHARCOAL)
        .set(ids.timeline, ui);

    /////////////////////////
    ///// PINNED TRACKS /////
    /////////////////////////
    //
    // Pin the ruler track to the top of the timeline.
    //
    // All pinned tracks must be `set` prior to non-pinned tracks.
    {
        let ruler = track::Ruler::new(context.ruler, &context.bars, PPQN).color(color);
        let track = context.set_next_pinned_track(ruler, ui);
        for triggered in track.event {
            *playhead_secs = triggered.ticks.ms(BPM, PPQN) / ONE_SECOND_MS;
        }
    }

    //////////////////
    ///// TRACKS /////
    //////////////////

    // Now that we've finished setting the pinned tracks, move on to the `Tracks` context.
    let context = context.start_tracks(ui);

    {
        // Piano roll.
        let piano_roll = track::PianoRoll::new(&context.bars, PPQN, &notes[..]).color(color);
        let track = context.set_next_track(piano_roll, ui);
        for event in track.event {
            use timeline::track::piano_roll::Event;
            match event {
                Event::NoteOn(_note_idx) => (),
                Event::NoteOff(_note_idx) => (),
                Event::NotePlayed(_note_idx) => (),
            }
        }

        // A macro for common logic between tempo and octave "numeric" envelopes.
        macro_rules! numeric_automation {
            ($envelope:expr) => {
                let track = {
                    let automation =
                        track::automation::Numeric::new(&context.bars, PPQN, $envelope)
                            .color(color);
                    context.set_next_track(automation, ui)
                };
                for event in track.event {
                    use timeline::track::automation::numeric::Event;
                    match event {
                        Event::Interpolate(_number) => (),
                        Event::Mutate(mutate) => mutate.apply($envelope),
                    }
                }
            };
        }

        // Tempo automation.
        numeric_automation!(tempo_envelope);
        // Octave automation.
        numeric_automation!(octave_envelope);

        // Toggle automation.
        let track = {
            let automation =
                track::automation::Toggle::new(&context.bars, PPQN, toggle_envelope).color(color);
            context.set_next_track(automation, ui)
        };
        for event in track.event {
            use timeline::track::automation::toggle::Event;
            match event {
                Event::Interpolate(_toggle) => (),
                Event::SwitchTo(_toggle) => (),
                Event::Mutate(mutate) => mutate.apply(toggle_envelope),
            }
        }

        // Bang automation.
        let track = {
            let automation =
                track::automation::Bang::new(&context.bars, PPQN, bang_envelope).color(color);
            context.set_next_track(automation, ui)
        };
        for event in track.event {
            use timeline::track::automation::bang::Event;
            match event {
                Event::Bang => println!("BANG!"),
                Event::Mutate(mutate) => mutate.apply(bang_envelope),
            }
        }
    }

    ////////////////////////////////
    ///// PLAYHEAD & SCROLLBAR /////
    ////////////////////////////////

    // Now that all tracks have been set, finish up and set the `Playhead` and `Scrollbar`.
    let context = context.end_tracks();

    // Set the playhead after all tracks have been set.
    for event in context.set_playhead(ui) {
        use timeline::playhead::Event;
        match event {
            Event::Pressed => println!("Playhead pressed!"),
            Event::DraggedTo(ticks) => *playhead_secs = ticks.ms(BPM, PPQN) / ONE_SECOND_MS,
            Event::Released => println!("Playhead released!"),
        }
    }

    // Set the scrollbar if it is visible.
    context.set_scrollbar(ui);
}
