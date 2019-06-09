use conrod::{self, widget, Colorable, Positionable, Sizeable, Widget};
use core::{self, time, BarIterator};
use playhead::{self, Playhead};
use ruler::Ruler;
use std;
use std::collections::HashMap;
use track;

/// A widget for viewing and controlling time related data.
#[derive(WidgetCommon)]
pub struct Timeline<B> {
    /// Data required by all conrod Widget types.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    /// Style information unique to Timeline.
    style: Style,
    /// The position of the playhead in ticks.
    playhead: Option<time::Ticks>,
    /// The duration of the timeline given as a list of Bars.
    bars: B,
    /// A height for tracks that haven't been given some uniquely specified height.
    maybe_track_height: Option<conrod::Scalar>,
}

/// All state to be cached within the Conrod `Ui` between updates.
pub struct State {
    ids: Ids,
    /// The position of the playhead in ticks.
    playhead: Option<time::Ticks>,
    /// State shared with the `Context` (used for setting tracks, playhead and scrollbar).
    shared: std::sync::Arc<std::sync::Mutex<Shared>>,
}

/// State shared between the `Timeline`'s cached `State` and the `Context` used for setting tracks.
#[derive(Debug)]
struct Shared {
    /// All track heights that have been overridden by manually dragging the separator.
    overridden_track_heights: HashMap<widget::Id, conrod::Scalar>,
    /// A map of the unique identifiers available for use for each type of `Track`.
    track_ids: HashMap<std::any::TypeId, Vec<widget::Id>>,
    /// A unique identifier for the separator that goes under each track.
    separator_ids: Vec<widget::Id>,
    /// The index of the next available `widget::Id` for each `Track` type.
    next_track_id_indices: HashMap<std::any::TypeId, usize>,
    /// The duration of the timeline given as a cached list of Bars.
    bars: Vec<core::Bar>,
}

widget_ids! {
    struct Ids {
        // The backdrop for all widgets whose kid area is the inner, non-bordered rect.
        canvas,
        // A subtle background reference line for each visible marker in the ruler.
        grid_lines[],
        // The scrollable surface upon which all non-pinned tracks are placed.
        scrollable_rectangle,
        // Scrollbar for the scrollable_rectangle.
        scrollbar,
        // If one is given, this is used for the `Playhead` widget.
        playhead,
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "theme.shape_color")]
    pub color: Option<conrod::Color>,
    #[conrod(default = "theme.border_width")]
    pub border: Option<conrod::Scalar>,
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<conrod::Color>,
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<conrod::Color>,
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<conrod::FontSize>,
    #[conrod(default = "theme.border_width")]
    pub separator_thickness: Option<conrod::Scalar>,
    #[conrod(default = "theme.border_color")]
    pub separator_color: Option<conrod::Color>,
}

/// Styling attributes for the tracks that make up the timeline.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TrackStyle {
    pub border: conrod::Scalar,
    pub label_color: conrod::Color,
    pub font_size: conrod::FontSize,
    pub color: conrod::Color,
    pub separator_thickness: conrod::Scalar,
    pub separator_color: conrod::Color,
    pub width: conrod::Scalar,
    pub maybe_height: Option<conrod::Scalar>,
}

/// A `Context` returned by the `Timeline` widget for setting tracks, `Playhead` and `Scrollbar`.
#[derive(Debug)]
pub struct Context {
    /// The list of musical `Bar`s that describes the temporal structure.
    ///
    /// To avoid unnecessary allocations, this `Vec` is "taken" from the `Timeline`'s `State` before
    /// the `Context` is returned. The `Vec` is then swapped back to the `Timeline`'s `State` when
    /// the `Context` is `drop`ped.
    pub bars: Vec<core::Bar>,
    /// The `Ruler` constructed by the `Timeline`.
    pub ruler: Ruler,
    /// Track-specific styling attributes.
    pub track_style: TrackStyle,

    /// The unique identifier used to instantiate the parent `Timeline` for this `Context`.
    pub timeline_id: widget::Id,
    /// The transparent upon which pinned tracks and the scrollable area are placed.
    pub canvas_id: widget::Id,
    /// The unique identifier for the scrollable canvas upon which tracks are placed.
    pub scrollable_rectangle_id: widget::Id,
    /// The unique identifier for the `Timeline`'s `Playhead` widget.
    pub playhead_id: widget::Id,
    /// The unique identifier for the `Timeline`'s `Scrollbar` widget.
    pub scrollbar_id: widget::Id,

    /// Whether or not the `Timeline` is scrollable.
    is_scrollable: bool,
    /// The index of the next track.
    track_index: std::cell::Cell<usize>,
    /// The height of all tracks and their separators combined.
    combined_track_height: std::cell::Cell<conrod::Scalar>,
    /// The playhead position in `Ticks` and the delta `Ticks` if a `Playhead` was given.
    maybe_playhead: Option<(time::Ticks, time::Ticks)>,

    /// State borrowed from the `Timeline`'s cached state.
    ///
    /// This will be `Some` for as long as the `Timeline`'s `State` exists within the `Ui`.
    shared: std::sync::Weak<std::sync::Mutex<Shared>>,
}

/// The first context stage returned by the `Timeline`.
///
/// Allows for setting pinned tracks which must be complete before setting non-pinned tracks.
#[derive(Debug)]
pub struct PinnedTracks {
    context: Context,
}

/// The timeline context stage that follows `PinnedTracks`.
///
/// Allows for setting tracks upon a scrollable area underneath the pinned tracks.
#[derive(Debug)]
pub struct Tracks {
    context: Context,
    next_non_pinned_track_index: std::cell::Cell<usize>,
    /// The height of all pinned tracks and their separators combined.
    pub pinned_tracks_height: conrod::Scalar,
}

/// The final timeline context stage. Follows the `Tracks` stage.
///
/// Allows for setting the `Playhead` and `Scrollbar` for the timeline.
pub struct Final {
    context: Context,
}

/// Information related to an instantiated `Track`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Track<E> {
    /// The unique identifier for the `Track`.
    pub id: widget::Id,
    /// The unique identifier for the `Track`'s separator (under the track).
    pub separator_id: widget::Id,
    /// The index of the `Track` within all tracks on the `Timeline`.
    pub index: usize,
    /// The index of the `Track` in relation to all sibling tracks.
    ///
    /// For non-pinned tracks, this represents the index starting from the first non-pinned track.
    pub sibling_index: usize,
    /// The event produced by the `Track`.
    pub event: E,
}

/// The thickness of the scrollbar.
pub const SCROLLBAR_THICKNESS: conrod::Scalar = 10.0;

impl Context {
    /// Instantiate the next `Track` in the `Timeline`'s list of tracks.
    ///
    /// The user never calls this directly. Instead, this method is called via
    /// `PinnedTracks::set_next_pinned_track` or `Tracks::set_next_track`.
    fn set_next_track<T>(
        &self,
        widget: T,
        parent_id: widget::Id,
        track_sibling_index: usize,
        ui: &mut conrod::UiCell,
    ) -> Track<T::Event>
    where
        T: track::Widget,
    {
        // Retrieve the state that is shared with the `Timeline`.
        let shared = self.shared.upgrade().expect(
            "No shared timeline state found. Check that the \
             `Ui` has not been dropped and that the \
             timeline's state has not been dropped from \
             the widget graph.",
        );
        let mut shared = shared.lock().unwrap();

        // Retrieve the index for this track within the list of all tracks.
        let track_index = self.track_index.get();

        // Retrieve the `widget::Id` to use for this `Track`.
        let track_id = {
            let type_id = std::any::TypeId::of::<T::State>();
            let index_of_next_id = {
                let index_of_next_id = shared.next_track_id_indices.entry(type_id).or_insert(0);
                let index = *index_of_next_id;
                *index_of_next_id += 1;
                index
            };

            // Check for an existing `widget::Id` that can be used.
            let existing_id = {
                let track_ids = shared.track_ids.entry(type_id).or_insert(Vec::new());
                track_ids.get(index_of_next_id).map(|&id| id)
            };

            match existing_id {
                Some(id) => id,
                None => {
                    // Create a new `widget::Id` for the track if there are none available.
                    let new_id = ui.widget_id_generator().next();
                    let track_ids = shared.track_ids.get_mut(&type_id).unwrap();
                    track_ids.push(new_id);
                    track_ids[index_of_next_id]
                }
            }
        };

        // Retrieve the `widget::Id` for the track separator that goes under this track.
        let separator_id = {
            while shared.separator_ids.len() <= track_index {
                shared.separator_ids.push(ui.widget_id_generator().next());
            }
            shared.separator_ids[track_index]
        };

        // Check to see whether the track separator has been moved and whether or not the
        // height of this track should be adjusted.
        if let Some(drag) = ui.widget_input(separator_id).drags().left().last() {
            let separator_y_range = ui.rect_of(separator_id).unwrap().y;
            let separator_h = separator_y_range.len();
            let half_separator_h = separator_h / 2.0;
            let y_top_max = match track_sibling_index {
                0 => ui.kid_area_of(parent_id).unwrap().top(),
                _ => ui
                    .rect_of(shared.separator_ids[track_index - 1])
                    .unwrap()
                    .bottom(),
            };
            let y_middle_max = y_top_max - half_separator_h;
            const MIN_TRACK_HEIGHT: conrod::Scalar = 1.0;
            let drag_y = separator_y_range.middle() + drag.to[1];
            let new_middle_y = drag_y.min(y_middle_max - MIN_TRACK_HEIGHT);
            let new_height = (y_middle_max + half_separator_h) - (new_middle_y - half_separator_h);
            shared.overridden_track_heights.insert(track_id, new_height);
        }

        // If the track does not yet have a specified height, check to see whether the `TrackStyle`
        // specifies some default height that should be used.
        let maybe_height = {
            shared
                .overridden_track_heights
                .get(&track_id)
                .map(|&h| h)
                .or_else(|| match widget.common().style.maybe_y_dimension {
                    None => self.track_style.maybe_height,
                    Some(_) => None,
                })
        };

        // Instantiate the track widget given by the user.
        let event = widget
            .and_then(self.maybe_playhead, |w, p| track::Widget::playhead(w, p))
            .w(self.track_style.width)
            .parent(parent_id)
            .and(|w| match track_sibling_index {
                0 => w.top_left_of(parent_id),
                _ => {
                    let last_separator_id = shared.separator_ids[track_index - 1];
                    w.down_from(last_separator_id, 0.0)
                }
            })
            .and_then(maybe_height, |w, h| w.h(h))
            .crop_kids()
            .set(track_id, ui);

        // The track separator. Goes underneath the track we are setting.
        {
            const MIN_HOVERED_SEPARATOR_H: conrod::Scalar = 6.0;
            const MIN_NEAR_SEPARATOR_H: conrod::Scalar = 3.0;

            // Expand the height of the separator slightly when the mouse is nearby.
            let separator_h = match ui.widget_input(separator_id).mouse() {
                Some(_) => self
                    .track_style
                    .separator_thickness
                    .max(MIN_HOVERED_SEPARATOR_H),
                None => match ui.global_input().current.widget_capturing_mouse {
                    Some(widget) => match ui.widget_input(track_id).mouse().is_some()
                        || ui
                            .widget_graph()
                            .does_recursive_depth_edge_exist(track_id, widget)
                    {
                        true => self
                            .track_style
                            .separator_thickness
                            .max(MIN_NEAR_SEPARATOR_H),
                        false => self.track_style.separator_thickness,
                    },
                    None => self.track_style.separator_thickness,
                },
            };

            // Highlight the separator when the mouse interacts with it.
            let separator_color = match ui.widget_input(separator_id).mouse() {
                Some(mouse) => match mouse.buttons.left().is_down() {
                    true => self.track_style.separator_color.clicked(),
                    false => self.track_style.separator_color.highlighted(),
                },
                None => self.track_style.separator_color,
            };

            widget::Rectangle::fill([self.track_style.width, separator_h])
                .down_from(track_id, 0.0)
                .parent(parent_id)
                .color(separator_color)
                .set(separator_id, ui);
        }

        let track_h = ui.h_of(track_id).unwrap();
        let separator_h = ui.h_of(separator_id).unwrap();
        self.combined_track_height
            .set(self.combined_track_height.get() + track_h + separator_h);
        self.track_index.set(track_index + 1);

        Track {
            id: track_id,
            separator_id: separator_id,
            index: track_index,
            sibling_index: track_sibling_index,
            event: event,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // When the `Context` is dropped, pass the list of `Bar`s back to the `Timeline` so that
        // the `Vec` may be re-used.
        if let Some(shared) = self.shared.upgrade() {
            if let Ok(mut shared) = shared.lock() {
                std::mem::swap(&mut shared.bars, &mut self.bars);
            }
        }
    }
}

impl PinnedTracks {
    /// Set the given `widget` as the next pinned track.
    ///
    /// Returns information about the `Track` as well as the `widget`'s `Event`.
    pub fn set_next_pinned_track<T>(&self, widget: T, ui: &mut conrod::UiCell) -> Track<T::Event>
    where
        T: track::Widget,
    {
        let parent_id = self.context.canvas_id;
        let sibling_track_index = self.context.track_index.get();
        self.context
            .set_next_track(widget, parent_id, sibling_track_index, ui)
    }

    /// Finalizes the `PinnedTracksContext` and returns a `TracksContext` that allows for setting
    /// regular, non-pinned tracks.
    pub fn start_tracks(self, ui: &mut conrod::UiCell) -> Tracks {
        let PinnedTracks { context } = self;

        // Place the scrollable canvas in the area underneath the pinned tracks.
        let inner_rect = ui
            .rect_of(context.timeline_id)
            .unwrap()
            .pad(context.track_style.border);
        let pinned_tracks_h = context.combined_track_height.get();
        let scrollable_rectangle_w = inner_rect.w();
        let scrollable_rectangle_h = inner_rect.h() - pinned_tracks_h;
        widget::Rectangle::fill([scrollable_rectangle_w, scrollable_rectangle_h])
            .color(conrod::color::TRANSPARENT)
            .bottom_left_of(context.canvas_id)
            .scroll_kids_vertically()
            .set(context.scrollable_rectangle_id, ui);

        Tracks {
            context: context,
            next_non_pinned_track_index: std::cell::Cell::new(0),
            pinned_tracks_height: pinned_tracks_h,
        }
    }
}

impl Tracks {
    /// Set the given `widget` as the next track.
    ///
    /// Returns information about the `Track` as well as the `widget`'s `Event`.
    pub fn set_next_track<T>(&self, widget: T, ui: &mut conrod::UiCell) -> Track<T::Event>
    where
        T: track::Widget,
    {
        let parent_id = self.context.scrollable_rectangle_id;
        let sibling_track_index = self.next_non_pinned_track_index.get();
        self.next_non_pinned_track_index
            .set(sibling_track_index + 1);
        self.context
            .set_next_track(widget, parent_id, sibling_track_index, ui)
    }

    /// To be called when all tracks have been instantiated.
    ///
    /// Returns a context which can be used to set the `Playhead` and `Scrollbar`.
    pub fn end_tracks(self) -> Final {
        let Tracks { context, .. } = self;
        Final { context: context }
    }
}

impl Final {
    /// Instantiate the `Playhead` widget, visible over all tracks that have been instantiated so
    /// far. To show the `Playhead` over all tracks, call this after all tracks have been set.
    pub fn set_playhead(&self, ui: &mut conrod::UiCell) -> Vec<playhead::Event> {
        let Final { ref context } = *self;

        let playhead = match context.maybe_playhead {
            Some((playhead, _)) => playhead,
            None => return Vec::new(),
        };

        // Get the position and height of the timeline widget.
        let timeline_rect = ui.rect_of(context.timeline_id).unwrap();

        const PLAYHEAD_WIDTH: conrod::Scalar = 6.0;
        let total_duration = context.bars.iter().cloned().total_duration();
        let clamped_playhead = conrod::utils::clamp(playhead, time::Ticks(0), total_duration);
        let playhead_weight = clamped_playhead.ticks() as f64 / total_duration.ticks() as f64;
        let half_combined_track_height = context.combined_track_height.get() / 2.0;
        let border = context.track_style.border;
        let y_offset = (timeline_rect.h() - border * 2.0) / 2.0 - half_combined_track_height;
        let playhead_y = timeline_rect.y() + y_offset;
        let left_of_timeline = timeline_rect.left() + border;
        let track_w = context.track_style.width;
        let x_from_left = playhead_weight * track_w;
        let playhead_x = left_of_timeline + x_from_left;
        let visible_tracks_x = conrod::Range::from_pos_and_len(timeline_rect.x(), track_w);
        let playhead_h = context.combined_track_height.get() - border * 2.0;

        Playhead::new(context.ruler, visible_tracks_x)
            .w_h(PLAYHEAD_WIDTH, playhead_h)
            .x_y(playhead_x, playhead_y)
            .color(context.track_style.color.complement())
            .parent(context.canvas_id)
            .set(context.playhead_id, ui)
    }

    /// If the timeline is scrollable, this sets the scrollbar on top.
    ///
    /// Returns the `widget::Id` of the scrollbar if it is currently scrollable.
    pub fn set_scrollbar(&self, ui: &mut conrod::UiCell) -> Option<widget::Id> {
        let Final { ref context } = *self;
        if context.is_scrollable {
            // Scrollbar for the scrollable canvas.
            let luminance = context.track_style.color.luminance();
            widget::Scrollbar::y_axis(context.scrollable_rectangle_id)
                .auto_hide(false)
                .thickness(SCROLLBAR_THICKNESS)
                .color(conrod::color::rgb(luminance, luminance, luminance))
                .set(context.scrollbar_id, ui);
            Some(context.scrollbar_id)
        } else {
            None
        }
    }
}

impl std::ops::Deref for PinnedTracks {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl std::ops::Deref for Tracks {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl std::ops::Deref for Final {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<B> Timeline<B> {
    /// Construct a new Timeline widget in it's default state.
    pub fn new(bars: B) -> Self
    where
        B: IntoIterator<Item = core::Bar>,
    {
        Timeline {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            playhead: None,
            bars: bars,
            maybe_track_height: None,
        }
    }

    builder_methods! {
        pub playhead { playhead = Some(time::Ticks) }
        pub track_height { maybe_track_height = Some(conrod::Scalar) }
        pub label_color { style.label_color = Some(conrod::Color) }
        pub label_font_size { style.label_font_size = Some(conrod::FontSize) }
        pub separator_thickness { style.separator_thickness = Some(conrod::Scalar) }
        pub separator_color { style.separator_color = Some(conrod::Color) }
    }
}

impl<B> conrod::Widget for Timeline<B>
where
    B: IntoIterator<Item = core::Bar>,
{
    type State = State;
    type Style = Style;
    type Event = PinnedTracks;

    fn init_state(&self, id_gen: widget::id::Generator) -> State {
        let shared = Shared {
            overridden_track_heights: HashMap::new(),
            track_ids: HashMap::new(),
            separator_ids: Vec::new(),
            next_track_id_indices: HashMap::new(),
            bars: Vec::new(),
        };
        State {
            ids: Ids::new(id_gen),
            playhead: None,
            shared: std::sync::Arc::new(std::sync::Mutex::new(shared)),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// The area of the widget below the title bar, upon which child widgets will be placed.
    fn kid_area(&self, args: widget::KidAreaArgs<Self>) -> widget::KidArea {
        let widget::KidAreaArgs {
            rect, style, theme, ..
        } = args;
        widget::KidArea {
            rect: rect.pad(style.border(theme) / 2.0),
            pad: conrod::position::Padding::none(),
        }
    }

    /// Update the state of the Timeline.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use conrod::{Borderable, Colorable, Positionable, Sizeable};
        use diff::{iter_diff, IterDiff};

        let widget::UpdateArgs {
            id,
            state,
            rect,
            style,
            ui,
            ..
        } = args;
        let Timeline {
            playhead,
            bars,
            maybe_track_height,
            ..
        } = self;
        let color = style.color(&ui.theme);
        let border = style.border(&ui.theme) / 2.0;
        let border_color = style.border_color(&ui.theme);
        let label_color = style.label_color(&ui.theme);
        let font_size = style.label_font_size(&ui.theme);
        let separator_thickness = style.separator_thickness(&ui.theme);
        let separator_color = style.separator_color(&ui.theme);
        let inner_rect = rect.pad(border);

        // The `shared` state is only ever shared with the `Context` which should not exist yet, so
        // it should be safe to unwrap.
        let temp_shared = state.shared.clone();
        let mut shared = temp_shared.lock().unwrap();

        // First, ensure that our `state`'s `bars` is up to date.
        if let Some(diff) = iter_diff(&shared.bars, bars) {
            match diff {
                IterDiff::FirstMismatch(i, bs) => {
                    shared.bars = shared.bars.iter().cloned().take(i).chain(bs).collect()
                }
                IterDiff::Longer(bs) => shared.bars.extend(bs),
                IterDiff::Shorter(len) => shared.bars.truncate(len),
            }
        }

        // Use a `Canvas` as the backdrop for the Tracks.
        widget::Canvas::new()
            .color(conrod::color::TRANSPARENT)
            .border_color(conrod::color::TRANSPARENT)
            .border(0.0)
            .middle_of(id)
            .wh_of(id)
            .pad(border)
            .set(state.ids.canvas, ui);

        // If the timeline is scrollable, adjust the width of the tracks to fit the Scrollbar.
        let is_scrollable = ui
            .widget_graph()
            .widget(state.ids.scrollable_rectangle)
            .and_then(|w| w.maybe_y_scroll_state.as_ref())
            .map(|scroll_state| scroll_state.offset_bounds.magnitude().is_sign_negative())
            .unwrap_or(false);

        // The scrollbar should not overlap with the tracks.
        let tracks_w = match is_scrollable {
            true => inner_rect.w() - SCROLLBAR_THICKNESS,
            false => inner_rect.w(),
        };

        // Construct the ruler for the Timeline in it's current state.
        let ruler = Ruler::new(tracks_w, shared.bars.iter().cloned());

        // Draw a light grid over the background to clarify ruler divisions.
        let total_ticks = shared.bars.iter().cloned().total_duration();
        let tracks_x = {
            let start = inner_rect.left();
            let end = start + tracks_w;
            conrod::position::Range::new(start, end)
        };

        // Ensure there are enough grid line `widget::Id`s.
        let num_markers = ruler.marker_count(shared.bars.iter().cloned());
        if state.ids.grid_lines.len() < num_markers {
            state.update(|state| {
                state
                    .ids
                    .grid_lines
                    .resize(num_markers, &mut ui.widget_id_generator());
            });
        }
        let mut grid_line_idx = 0;
        for bar_markers in ruler.markers_in_ticks(shared.bars.iter().cloned()) {
            for (i, ticks) in bar_markers.enumerate() {
                let x_offset = super::ruler::x_offset_from_ticks(ticks, total_ticks, tracks_w);
                let line_x = tracks_x.middle() + x_offset;
                let a = [line_x, inner_rect.top()];
                let b = [line_x, inner_rect.bottom()];
                let color = match i {
                    0 => border_color.alpha(0.5),
                    _ => border_color.alpha(0.125),
                };
                let line_id = state.ids.grid_lines[grid_line_idx];
                widget::Line::abs(a, b)
                    .graphics_for(state.ids.canvas)
                    .parent(state.ids.canvas)
                    .color(color)
                    .thickness(1.0)
                    .set(line_id, ui);
                grid_line_idx += 1;
            }
        }

        let maybe_playhead = playhead.map(|playhead| {
            let delta = state
                .playhead
                .map(|old| playhead - old)
                .unwrap_or(time::Ticks(0));
            (playhead, delta)
        });

        // Reset all the next track indices to `0`.
        for index in shared.next_track_id_indices.values_mut() {
            *index = 0;
        }

        if state.playhead != playhead {
            state.update(|state| state.playhead = playhead);
        }

        let track_style = TrackStyle {
            border: border,
            label_color: label_color,
            font_size: font_size,
            color: color,
            separator_thickness: separator_thickness,
            separator_color: separator_color,
            width: tracks_w,
            maybe_height: maybe_track_height,
        };

        let context = Context {
            bars: std::mem::replace(&mut shared.bars, Vec::new()),
            ruler: ruler,
            track_style: track_style,
            shared: std::sync::Arc::downgrade(&state.shared),
            timeline_id: id,
            scrollable_rectangle_id: state.ids.scrollable_rectangle,
            playhead_id: state.ids.playhead,
            scrollbar_id: state.ids.scrollbar,
            canvas_id: state.ids.canvas,
            is_scrollable: is_scrollable,
            track_index: std::cell::Cell::new(0),
            combined_track_height: std::cell::Cell::new(0.0),
            maybe_playhead: maybe_playhead,
        };

        PinnedTracks { context: context }
    }
}

impl<B> conrod::Colorable for Timeline<B> {
    builder_method!(color { style.color = Some(conrod::Color) });
}

impl<B> conrod::Borderable for Timeline<B> {
    builder_methods! {
        border { style.border = Some(conrod::Scalar) }
        border_color { style.border_color = Some(conrod::Color) }
    }
}
