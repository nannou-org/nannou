use bars_duration_ticks;
use conrod_core::{self as conrod, widget};
use period::Period;
use track;
use pitch_calc::{self as pitch, Letter, LetterOctave};
use time_calc::{self as time, Ticks};

/// A PianoRoll widget builder type.
#[derive(WidgetCommon)]
pub struct PianoRoll<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    bars: &'a [time::TimeSig],
    ppqn: time::Ppqn,
    /// The current position of the playhead and the amount it has changed respectively.
    maybe_playhead: Option<(Ticks, Ticks)>,
    notes: &'a [Note],
    style: Style,
}

/// Used to represent a musical note within the piano roll.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Note {
    /// The period over which the note is played.
    pub period: Period,
    /// The pitch of the note.
    pub pitch: LetterOctave,
}

/// An alias for an index into the note buffer.
pub type NoteIdx = usize;

pub const MIN_NOTE_TRACK_HEIGHT: conrod::Scalar = 10.0;

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "theme.shape_color")]
    pub color: Option<conrod::Color>,
}

widget_ids! {
    struct Ids {
        scrollable_rectangle,
        notes[],
        black_key_tracks[],
    }
}

/// A track for presenting pattern information.
pub struct State {
    ids: Ids,
}

/// The various kinds of Events that may occur with the PianoRoll track.
#[derive(Copy, Clone, Debug)]
pub enum Event {
    /// The playhead passed over the start of the node at the index.
    NoteOn(NoteIdx),
    /// The playhead passed over the end of the node at the index.
    NoteOff(NoteIdx),
    /// The playhead passed over some part of the node at the index.
    NotePlayed(NoteIdx),
}

impl<'a> PianoRoll<'a> {
    /// Construct a new, default PianoRoll.
    pub fn new(bars: &'a [time::TimeSig], ppqn: time::Ppqn, notes: &'a [Note]) -> Self {
        PianoRoll {
            bars: bars,
            ppqn: ppqn,
            maybe_playhead: None,
            notes: notes,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
        }
    }
}

impl<'a> track::Widget for PianoRoll<'a> {
    fn playhead(mut self, playhead: (Ticks, Ticks)) -> Self {
        self.maybe_playhead = Some(playhead);
        self
    }
}

impl<'a> conrod::Widget for PianoRoll<'a> {
    type State = State;
    type Style = Style;
    type Event = Vec<Event>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    // If a height was not specified, we'll determine the height by summing the height of the note
    // tracks together at their minimum height.
    fn default_y_dimension(&self, _ui: &conrod::Ui) -> conrod::position::Dimension {
        const MIN_DEFAULT_TRACK_HEIGHT: conrod::Scalar = 70.0;
        let (min_step, max_step) = note_step_range(self.notes);
        let num_steps_in_range = (max_step - min_step + 1.0).ceil();
        let height = num_steps_in_range as conrod::Scalar * MIN_NOTE_TRACK_HEIGHT;
        conrod::position::Dimension::Absolute(height.max(MIN_DEFAULT_TRACK_HEIGHT))
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use conrod_core::{Borderable, Colorable, Positionable};

        let widget::UpdateArgs {
            id,
            rect,
            state,
            style,
            ui,
            ..
        } = args;

        // let visible_height = self.get_visible_height(&ui);
        let PianoRoll {
            bars,
            ppqn,
            maybe_playhead,
            notes,
            ..
        } = self;

        let (min_step, max_step) = note_step_range(self.notes);
        // let note_track_height = note_track_height(visible_height, min_step, max_step);
        let note_track_height = note_track_height(rect.h(), min_step, max_step);

        // Determine the range of notes over which the playhead has moved.
        let playhead_delta_range = match maybe_playhead {
            Some((playhead, delta)) if delta > Ticks(0) => {
                let start = playhead - delta;
                let end = playhead;
                let predicate = |n: &Note| n.period.contains(start) || n.period.contains(end);
                let maybe_start_idx = notes.iter().position(&predicate);
                let maybe_end_idx = notes.iter().rposition(&predicate);
                match (maybe_start_idx, maybe_end_idx) {
                    (Some(start_idx), Some(end_idx)) => Some((start_idx, end_idx)),
                    _ => None,
                }
            }
            _ => None,
        };

        let mut events = Vec::new();

        // If the playhead passed over some note(s), react accordingly.
        if let Some((start_idx, end_idx)) = playhead_delta_range {
            if let Some((playhead, delta)) = maybe_playhead {
                let playhead_range = Period {
                    start: playhead - delta,
                    end: playhead,
                };
                for i in start_idx..end_idx {
                    if playhead_range.contains(notes[i].period.start) {
                        events.push(Event::NoteOn(i))
                    }
                    if playhead_range.contains(notes[i].period.end) {
                        events.push(Event::NoteOff(i))
                    }
                    if playhead_range.intersects(&notes[i].period) {
                        events.push(Event::NotePlayed(i))
                    }
                }
            }
        }

        // All that remains is to instantiate the graphics widgets.
        //
        // Check whether or not we need to do so by checking whether or not we're visible.
        if conrod::graph::algo::cropped_area_of_widget(ui.widget_graph(), id).is_none() {
            return events;
        }

        // Start instantiating the graphics widgets.
        let (x, y, w, h) = rect.x_y_w_h();
        let color = style.color(ui.theme());

        // Instantiate the scrollable rectangle upon which the notes and tracks will be placed.
        widget::Rectangle::fill([w, h])
            .x_y(x, y)
            .parent(id)
            .color(conrod::color::TRANSPARENT)
            .scroll_kids_vertically()
            .set(state.ids.scrollable_rectangle, ui);

        fn is_black_key_step(step: &i32) -> bool {
            pitch::Step(*step as f32).letter().is_black_key()
        }

        // We'll draw the tracks from the bottom up.
        let start_step = min_step.floor() as i32;
        let end_step = (max_step + 1.0).floor() as i32;
        let bottom_track_y_offset = note_track_height / 2.0 - h / 2.0;
        let num_note_tracks = (start_step..end_step).filter(is_black_key_step).count();

        // Before we go on, check we have enough `NoteIndex`s in our state.
        if state.ids.black_key_tracks.len() < num_note_tracks {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.black_key_tracks.resize(num_note_tracks, id_gen));
        }

        // Only draw the note tracks if there is more than one note track visible.
        if num_note_tracks > 1 {
            let note_track_color = color
                .plain_contrast()
                .plain_contrast()
                .highlighted()
                .alpha(0.075);
            let iter = (start_step..end_step)
                .enumerate()
                .filter(|&(_, step)| is_black_key_step(&step))
                .zip(state.ids.black_key_tracks.iter());
            for ((i, _), &black_key_track_id) in iter {
                let y_offset = bottom_track_y_offset + i as conrod::Scalar * note_track_height;
                widget::Rectangle::fill([w, note_track_height])
                    .y_relative_to(id, y_offset)
                    .parent(state.ids.scrollable_rectangle)
                    .color(note_track_color)
                    .set(black_key_track_id, ui);
            }
        }

        // Check that we have at least one `NodeIndex` for each note.
        if state.ids.notes.len() < notes.len() {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.notes.resize(notes.len(), id_gen));
        }

        // Instantiate a **Rectangle** for each note on our PianoRoll.
        let total_ticks = bars_duration_ticks(bars.iter().cloned(), ppqn);
        let half_w = w / 2.0;

        // Converts the given position along the timeline in ticks to an x_offset.
        let ticks_to_x_offset = move |ticks: Ticks| -> conrod::Scalar {
            (ticks.ticks() as conrod::Scalar / total_ticks.ticks() as conrod::Scalar) * w
        };

        // Converts the period along the timeline to a Scalar Range.
        let period_to_w_and_x_offset =
            move |period: Period| -> (conrod::Scalar, conrod::Scalar) {
                let half_duration = Ticks(period.duration().ticks() / 2);
                let period_middle = period.start + half_duration;
                let middle_x_offset = ticks_to_x_offset(period_middle) - half_w;
                let width = ticks_to_x_offset(period.duration());
                (width, middle_x_offset)
            };

        let iter = notes.iter().enumerate().zip(state.ids.notes.iter());
        for ((i, note), &note_id) in iter {
            let step = note.pitch.step();
            let steps_from_bottom = step as conrod::Scalar - start_step as conrod::Scalar;
            let y_offset = bottom_track_y_offset + steps_from_bottom * note_track_height;
            let (w, x_offset) = period_to_w_and_x_offset(note.period);
            let note_color = match playhead_delta_range {
                Some((start, end)) if i >= start && i <= end => color.clicked(),
                _ => color,
            };
            let note_border_color = note_color.plain_contrast();
            widget::BorderedRectangle::new([w, note_track_height])
                .border(1.0)
                .border_color(note_border_color)
                .x_y_relative_to(id, x_offset, y_offset)
                .parent(state.ids.scrollable_rectangle)
                .color(note_color)
                .set(note_id, ui);
        }

        events
    }
}

/// A single note range at C 1.
fn default_step_range() -> (pitch::calc::Step, pitch::calc::Step) {
    let c_1_step = LetterOctave(Letter::C, 1).step();
    (c_1_step, c_1_step)
}

/// The lowest and heighest pitch notes found within the given slice of notes.
fn note_step_range(notes: &[Note]) -> (pitch::calc::Step, pitch::calc::Step) {
    // Determine the lowest and highest pitches in steps.
    if notes.len() > 0 {
        let init = (::std::f32::MAX, ::std::f32::MIN);
        notes.iter().fold(init, |(lowest, highest), note| {
            let step = note.pitch.step();
            (lowest.min(step), highest.max(step))
        })
    } else {
        // If there are no notes to display, we'll make up a single note range.
        default_step_range()
    }
}

/// Determine the height to be used for the note tracks.
fn note_track_height(
    visible_height: conrod::Scalar,
    min_step: pitch::calc::Step,
    max_step: pitch::calc::Step,
) -> conrod::Scalar {
    // Define the bounds for the height of a displayed `Step`.
    let max_note_track_height: conrod::Scalar = visible_height;

    // Determine the inclusive length of the note step range.
    let num_steps_in_range = (max_step - min_step) + 1.0;
    let height_per_step_for_range = visible_height / num_steps_in_range as f64;

    // Ensure that the height_per_step is within the min and max range.
    height_per_step_for_range
        .max(MIN_NOTE_TRACK_HEIGHT)
        .min(max_note_track_height)
}

impl<'a> conrod::Colorable for PianoRoll<'a> {
    builder_method!(color { style.color = Some(conrod::Color) });
}
