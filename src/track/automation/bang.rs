use bars_duration_ticks;
use conrod_core::{self as conrod, widget};
use env;
use ruler;
use super::EnvelopeTrait;
use time_calc::{self as time, Ticks};
use track;

pub use env::{Bang as BangValue, Point, PointTrait};

/// The bounded envelope type compatible with the `Bang` automation track.
pub type Envelope = env::bounded::Envelope<BangValue>;

/// For viewing and manipulating a series of discrete points over time.
#[derive(WidgetCommon)]
pub struct Bang<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    envelope: &'a Envelope,
    bars: &'a [time::TimeSig],
    ppqn: time::Ppqn,
    /// The position of the playhead in ticks, along with its change in position.
    pub maybe_playhead: Option<(Ticks, Ticks)>,
    style: Style,
}

/// Unique state for the Bang automation.
pub struct State {
    /// A `Pole` (collection of indices) for each point in the envelope.
    point_poles: Vec<Pole>,
    /// The `Pole` used to instantiate the widgets for the phantom cursor point.
    phantom_pole: Pole,
    // /// A NodeIndex for drawing the value the point closest to the cursor using a Text widget.
    // ///
    // /// TODO: use this to draw (Bar, Measure, Ticks) string to closest point.
    // closest_point_ticks_text_idx: IndexSlot,
}

/// The indices necessary for a single envelope point `Pole` representation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pole {
    top_circle_id: widget::Id,
    bottom_circle_id: widget::Id,
    line_id: widget::Id,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "theme.shape_color")]
    pub color: Option<conrod::Color>,
    #[conrod(default = "4.0")]
    pub point_radius: Option<conrod::Scalar>,
}

/// The various kinds of events returned by an automation track.
#[derive(Copy, Clone, Debug)]
pub enum Event {
    /// Occurs if the playhead has passed a `Bang`.
    Bang,
    /// Mutate an envelope point in some manner.
    Mutate(super::Mutate<BangValue>),
}

impl<'a> Bang<'a> {
    /// Construct a new default Automation.
    pub fn new(bars: &'a [time::TimeSig], ppqn: time::Ppqn, envelope: &'a Envelope) -> Self {
        Bang {
            bars: bars,
            ppqn: ppqn,
            maybe_playhead: None,
            envelope: envelope,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
        }
    }

    /// The Automation with some given point radius to use for the envelope points.
    pub fn point_radius(mut self, radius: conrod::Scalar) -> Self {
        self.style.point_radius = Some(radius);
        self
    }
}

impl<'a> track::Widget for Bang<'a> {
    fn playhead(mut self, playhead: (Ticks, Ticks)) -> Self {
        self.maybe_playhead = Some(playhead);
        self
    }
}

impl Pole {
    pub fn new(id_gen: &mut widget::id::Generator) -> Self {
        Pole {
            top_circle_id: id_gen.next(),
            bottom_circle_id: id_gen.next(),
            line_id: id_gen.next(),
        }
    }
}

impl<'a> conrod::Colorable for Bang<'a> {
    fn color(mut self, color: conrod::Color) -> Self {
        self.style.color = Some(color);
        self
    }
}

impl<'a> conrod::Widget for Bang<'a> {
    type State = State;
    type Style = Style;
    type Event = Vec<Event>;

    fn init_state(&self, mut id_gen: widget::id::Generator) -> Self::State {
        State {
            point_poles: Vec::new(),
            phantom_pole: Pole::new(&mut id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_y_dimension(&self, ui: &conrod::Ui) -> conrod::position::Dimension {
        ui.theme
            .widget_style::<Style>()
            .and_then(|default| default.common.maybe_y_dimension)
            .unwrap_or(conrod::position::Dimension::Absolute(
                super::super::DEFAULT_HEIGHT,
            ))
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use super::Elem;
        use conrod_core::{Colorable, Positionable};

        let widget::UpdateArgs {
            id,
            rect,
            state,
            style,
            ui,
            ..
        } = args;
        let Bang {
            envelope,
            bars,
            ppqn,
            maybe_playhead,
            ..
        } = self;

        // Ensure we've a `Pole` for each point in the envelope.
        let num_points = envelope.points().count();
        if state.point_poles.len() < num_points {
            state.update(|state| {
                let current_len = state.point_poles.len();
                let id_gen = &mut ui.widget_id_generator();
                let extension = (current_len..num_points).map(|_| Pole::new(id_gen));
                state.point_poles.extend(extension);
            });
        }

        let (x, _, w, h) = rect.x_y_w_h();
        let half_h = h / 2.0;
        let color = style.color(ui.theme());
        let point_radius = style.point_radius(ui.theme());
        const THICKNESS: conrod::Scalar = 2.0;
        let total_ticks = bars_duration_ticks(bars.iter().cloned(), ppqn);

        // Get the x position of the given ticks relative to the centre of the Bang automation.
        let rel_x_from_ticks =
            |ticks: Ticks| -> conrod::Scalar { ruler::x_offset_from_ticks(ticks, total_ticks, w) };

        // Get the absolute x value from the given ticks.
        let x_from_ticks = |ticks: Ticks| -> conrod::Scalar { rel_x_from_ticks(ticks) + x };

        // Get the time in ticks from some position over the Bang automation.
        let ticks_from_x = |x: conrod::Scalar| -> Ticks {
            Ticks(conrod::utils::map_range(
                x,
                rect.left(),
                rect.right(),
                0,
                total_ticks.ticks(),
            ))
        };

        // Same as `ticks_from_x` but clamps the ticks to the total_ticks range.
        let clamped_ticks_from_x = |x: conrod::Scalar| -> Ticks {
            conrod::utils::clamp(ticks_from_x(x), Ticks(0), total_ticks)
        };

        // All that remains is to instantiate the graphics widgets.
        //
        // Check whether or not we need to do so by checking whether or not we're visible.
        if conrod::graph::algo::cropped_area_of_widget(ui.widget_graph(), id).is_none() {
            return Vec::new();
        }

        // Determine the element range over which the playhead has traversed since the last update.
        let playhead_delta_range = match maybe_playhead {
            Some((playhead, delta)) if delta > Ticks(0) => {
                let start = playhead - delta;
                let end = playhead;
                super::maybe_surrounding_elems(total_ticks, envelope, start, end)
            }
            _ => None,
        };

        let bottom_y = rect.bottom();
        let top_y = rect.top();

        // A function for instantiating the necessary widgets for a point.
        let point_widgets = |pole: Pole,
                             color: conrod::Color,
                             thickness: conrod::Scalar,
                             x: conrod::Scalar,
                             ui: &mut conrod::UiCell,
                             is_graphic: bool| {
            let start = [x, bottom_y];
            let end = [x, top_y];
            widget::Line::abs(start, end)
                .and_if(is_graphic, |l| l.graphics_for(id))
                .color(color)
                .parent(id)
                .thickness(thickness)
                .set(pole.line_id, ui);

            let circle = widget::Circle::fill(point_radius)
                .color(color)
                .graphics_for(pole.line_id)
                .parent(pole.line_id);

            // The circle at the bottom of the line.
            circle
                .clone()
                .y_relative_to(pole.line_id, -half_h)
                .set(pole.bottom_circle_id, ui);

            // The circle at the top of the line.
            circle
                .y_relative_to(pole.line_id, half_h)
                .set(pole.top_circle_id, ui);
        };

        let mut events = Vec::new();

        // Instantiate the widgets for each point.
        let points = envelope.points().enumerate();
        let iter = state.point_poles.iter().zip(points);
        for (&pole, (i, &point)) in iter {
            let x = x_from_ticks(point.ticks);

            // Check for events received by the `Bang`.
            for widget_event in ui.widget_input(pole.line_id).events() {
                use conrod_core::{event, input};
                match widget_event {
                    // Check to see whether or not a point was dragged.
                    event::Widget::Drag(drag) if drag.button == input::MouseButton::Left => {
                        let line_rect = ui.rect_of(pole.line_id).unwrap();
                        let drag_to_abs_x = drag.to[0] + line_rect.x();
                        let drag_point = super::DragPoint {
                            idx: i,
                            ticks: clamped_ticks_from_x(drag_to_abs_x),
                            value: BangValue,
                        };
                        events.push(Event::Mutate(drag_point.into()));
                    }

                    // If the point was clicked with a right mouse button, remove it.
                    event::Widget::Click(click) if click.button == input::MouseButton::Right => {
                        let remove_point = super::RemovePoint { idx: i };
                        events.push(Event::Mutate(remove_point.into()));
                    }

                    _ => (),
                }
            }

            // Color the point via the playhead.
            let color = match playhead_delta_range {
                Some((start_elem, end_elem)) => {
                    let start = match start_elem {
                        Elem::Point(start) => Some(start),
                        Elem::BetweenPoints(_, start) => Some(start),
                        _ => None,
                    };
                    let end = match end_elem {
                        Elem::Point(end) => Some(end),
                        Elem::BetweenPoints(end, _) => Some(end),
                        _ => None,
                    };
                    match (start, end) {
                        (Some(start), Some(end)) if i >= start && i <= end => color.clicked(),
                        _ => color,
                    }
                }
                None => color,
            };

            let (color, thickness) = match ui.widget_input(pole.line_id).mouse() {
                Some(mouse) => match mouse.buttons.left().is_down() {
                    false => (color.highlighted(), THICKNESS + 1.0),
                    true => (color.clicked(), THICKNESS + 1.0),
                },
                None => (color, THICKNESS),
            };

            point_widgets(pole, color, thickness, x, ui, false);
        }

        // Check to see whether or not a new point should be added.
        for click in ui.widget_input(id).clicks().left() {
            let click_x = click.xy[0] + rect.x();
            let point = env::Point {
                ticks: clamped_ticks_from_x(click_x),
                value: BangValue,
            };
            let add_point = super::AddPoint { point: point };
            events.push(Event::Mutate(add_point.into()));
        }

        // If the mouse is over the widget, check if we should draw the phantom point `Pole`.
        if let Some((mouse_abs_x, is_left_down)) = ui
            .widget_input(id)
            .mouse()
            .map(|m| (m.abs_xy()[0], m.buttons.left().is_down()))
        {
            // Check if the playhead should affect the color of the phantom point.
            let ticks = clamped_ticks_from_x(mouse_abs_x);
            let playhead_passed_over = match maybe_playhead {
                Some((playhead, delta)) => {
                    delta > Ticks(0) && (playhead - delta) < ticks && ticks < playhead
                }
                None => false,
            };
            let color = if playhead_passed_over {
                color.clicked()
            } else {
                color
            };
            let color = match is_left_down {
                false => color.highlighted().alpha(0.25),
                true => color.clicked().alpha(0.5),
            };

            point_widgets(state.phantom_pole, color, THICKNESS, mouse_abs_x, ui, true);
        }

        events
    }
}
