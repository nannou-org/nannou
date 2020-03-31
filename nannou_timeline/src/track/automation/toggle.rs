use bars_duration_ticks;
use conrod_core::{self as conrod, widget};
use env;
use ruler;
use time_calc::{self as time, Ticks};
use track;

pub use env::{Point, PointTrait, Toggle as ToggleValue, Trait as EnvelopeTrait};

/// The envelope type compatible with the `Toggle` automation track.
pub type Envelope = env::bounded::Envelope<ToggleValue>;

/// For viewing and manipulating series of boolean valued points over time.
#[derive(WidgetCommon)]
pub struct Toggle<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    envelope: &'a Envelope,
    bars: &'a [time::TimeSig],
    ppqn: time::Ppqn,
    /// The position of the playhead in ticks along with the change in its position in ticks.
    pub maybe_playhead: Option<(Ticks, Ticks)>,
    style: Style,
}

/// Unique state for the Toggle automation.
pub struct State {
    ids: Ids,
}

widget_ids! {
    struct Ids {
        circles[],
        rectangles[],
        phantom_line,
    }
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
    /// Upon playhead movement, represents new boolean value at playhead.
    Interpolate(bool),
    /// Indicatees that the toggle value has changed since the last update.
    SwitchTo(bool),
    /// Some event which would mutate the envelope has occurred.
    Mutate(super::Mutate<ToggleValue>),
}

impl<'a> Toggle<'a> {
    /// Construct a new default Automation.
    pub fn new(bars: &'a [time::TimeSig], ppqn: time::Ppqn, envelope: &'a Envelope) -> Self {
        Toggle {
            bars: bars,
            ppqn: ppqn,
            maybe_playhead: None,
            envelope: envelope,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
        }
    }

    builder_methods! {
        pub point_radius { style.point_radius = Some(conrod::Scalar) }
    }
}

impl<'a> track::Widget for Toggle<'a> {
    fn playhead(mut self, playhead: (Ticks, Ticks)) -> Self {
        self.maybe_playhead = Some(playhead);
        self
    }
}

impl<'a> conrod::Colorable for Toggle<'a> {
    builder_method!(color { style.color = Some(conrod::Color) });
}

impl<'a> conrod::Widget for Toggle<'a> {
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
        use conrod_core::utils::{clamp, map_range};
        use conrod_core::{Colorable, Positionable};

        let widget::UpdateArgs {
            id,
            rect,
            state,
            style,
            ui,
            ..
        } = args;
        let Toggle {
            envelope,
            bars,
            ppqn,
            maybe_playhead,
            ..
        } = self;

        let num_points = envelope.points().count();
        let num_rectangles = {
            let mut points = envelope.points();
            points
                .next()
                .map(|first| {
                    let mut prev_toggle = first.value;
                    let mut count = 0;
                    for point in points {
                        if prev_toggle == ToggleValue(true) {
                            count += 1;
                        }
                        prev_toggle = point.value;
                    }
                    count
                })
                .unwrap_or(0)
        };

        // Ensure we have a circle index for each point.
        if state.ids.circles.len() < num_points {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.circles.resize(num_points, id_gen));
        }

        // Ensure we have a rectangle index for each point.
        if state.ids.rectangles.len() < num_rectangles {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.rectangles.resize(num_rectangles, id_gen));
        }

        let (w, h) = rect.w_h();
        let half_h = h / 2.0;
        let color = style.color(ui.theme());
        let point_radius = style.point_radius(ui.theme());
        let total_ticks = bars_duration_ticks(bars.iter().cloned(), ppqn);

        // Get the time in ticks from some position over the Bang automation.
        let ticks_from_x = |x: conrod::Scalar| {
            Ticks(map_range(
                x,
                rect.left(),
                rect.right(),
                0,
                total_ticks.ticks(),
            ))
        };
        // `false` if `y` is closer to the bottom, `true` if y is closer to the top.
        let value_from_y = |y: conrod::Scalar| {
            let perc = map_range(y, rect.bottom(), rect.top(), 0.0, 1.0);
            if perc < 0.5 {
                ToggleValue(false)
            } else {
                ToggleValue(true)
            }
        };

        // Same as `ticks_from_x` but clamps the ticks to the total_ticks range.
        let clamped_ticks_from_x =
            |x: conrod::Scalar| clamp(ticks_from_x(x), Ticks(0), total_ticks);

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

        // A function for instantiating a Circle widget for a point.
        let point_widget = |i: usize,
                            x_offset: conrod::Scalar,
                            value: ToggleValue,
                            point_id: widget::Id,
                            ui: &mut conrod::UiCell,
                            events: &mut Vec<Event>| {
            for widget_event in ui.widget_input(point_id).events() {
                use conrod_core::{event, input};

                match widget_event {
                    // Check to see if the toggle point is being dragged.
                    event::Widget::Drag(drag) if drag.button == input::MouseButton::Left => {
                        let point_rect = ui.rect_of(point_id).unwrap();
                        let drag_to_abs_xy = conrod::utils::vec2_add(drag.to, point_rect.xy());
                        let drag_point = super::DragPoint {
                            idx: i,
                            ticks: clamped_ticks_from_x(drag_to_abs_xy[0]),
                            value: value_from_y(drag_to_abs_xy[1]),
                        };
                        events.push(Event::Mutate(drag_point.into()));
                    }

                    // Check to see if the toggle point is being removed.
                    event::Widget::Click(click) if click.button == input::MouseButton::Right => {
                        let remove_point = super::RemovePoint { idx: i };
                        events.push(Event::Mutate(remove_point.into()));
                    }

                    _ => (),
                }
            }

            let y_offset = if value == ToggleValue(false) {
                -half_h
            } else {
                half_h
            };
            let point_elem = Elem::Point(i);
            let color = super::color_elem_by_playhead(point_elem, playhead_delta_range, color);

            let color = match ui.widget_input(point_id).mouse() {
                Some(mouse) => match mouse.buttons.left().is_down() {
                    true => color.clicked(),
                    false => color.highlighted(),
                },
                None => color,
            };
            widget::Circle::fill(point_radius)
                .x_y_relative_to(id, x_offset, y_offset)
                .graphics_for(id)
                .parent(id)
                .color(color)
                .set(point_id, ui);
        };

        let mut events = Vec::new();

        // Instantiate the widgets in a big loop.
        let mut iter = envelope.points().zip(state.ids.circles.iter()).enumerate();
        if let Some((i, (&first, &first_id))) = iter.next() {
            // The first point widget.
            let first_offset = ruler::x_offset_from_ticks(first.ticks, total_ticks, w);
            point_widget(i, first_offset, first.value, first_id, ui, &mut events);

            let mut prev_offset = first_offset;
            let mut prev_toggle = first.value;
            let mut rectangle_ids = state.ids.rectangles.iter();
            let mut prev_point_id = first_id;
            for (i, (&point, &point_id)) in iter {
                // All following point widgets.
                let point_x_offset = ruler::x_offset_from_ticks(point.ticks, total_ticks, w);
                point_widget(i, point_x_offset, point.value, point_id, ui, &mut events);

                // The rectangle widget.
                if prev_toggle == ToggleValue(true) {
                    let &rectangle_id = rectangle_ids.next().expect("Not enough rectangle ids");
                    let right = point_x_offset;
                    let left = prev_offset;
                    let width = right - left;
                    let elem = Elem::BetweenPoints(i - 1, i);
                    let color = super::color_elem_by_playhead(elem, playhead_delta_range, color);

                    let color = match ui
                        .widget_input(prev_point_id)
                        .mouse()
                        .or_else(|| ui.widget_input(point_id).mouse())
                    {
                        Some(mouse) => match mouse.buttons.left().is_down() {
                            true => color.clicked(),
                            false => color.highlighted(),
                        },
                        None => color,
                    };

                    let x_offset = left + width / 2.0;
                    widget::Rectangle::fill([width, h])
                        .depth(2.0) // Place behind lines and circles.
                        .x_relative_to(id, x_offset)
                        .graphics_for(id)
                        .color(color.alpha(0.5))
                        .parent(id)
                        .set(rectangle_id, ui);
                }

                prev_offset = point_x_offset;
                prev_toggle = point.value;
                prev_point_id = point_id;
            }
        }

        // // A Line widget to accent the current interaction with the widget.
        // if let Some(mouse) = ui.widget_input(idx).mouse() {

        //     let (x, ticks, value) = match new_interaction {
        //         Highlighted(Elem::Point(p_idx)) | Clicked(Elem::Point(p_idx), _, _) => {
        //             let p = envelope.env.points[p_idx];
        //             let x = x_from_ticks(p.ticks);
        //             (x, p.ticks, p.value)
        //         },
        //         Highlighted(_) | Clicked(_, _, _) => {
        //             let x = mouse.xy[0];
        //             let ticks = ticks_from_x(x);
        //             let value = value_from_y(mouse.xy[1]);
        //             (x, ticks, value)
        //         },
        //         _ => return,
        //     };

        //     let color = match new_interaction {

        //         // If whatever we're interacting with is highlighted, we should be too.
        //         Highlighted(Elem::Point(_)) => color.highlighted(),
        //         Highlighted(_) => color.highlighted().alpha(0.5),

        //         // Only draw the clicked point if it is still between the clicked area.
        //         Clicked(Elem::BetweenPoints(a, b), _, _) =>
        //             match (envelope.points().nth(a), envelope.points().nth(b)) {
        //                 (Some(p_a), Some(p_b)) if p_a.ticks <= ticks && ticks <= p_b.ticks =>
        //                     color.clicked().alpha(0.7),
        //                 _ => return,
        //             },

        //         // Only draw the clicked point if it is still before the first point.
        //         Clicked(Elem::BeforeFirstPoint, _, _) =>
        //             match envelope.points().nth(0) {
        //                 Some(p) if ticks <= p.ticks => color.clicked().alpha(0.7),
        //                 _ => return,
        //             },

        //         // Only draw the clicked point if it is still after the last point.
        //         Clicked(Elem::AfterLastPoint, _, _) =>
        //             match envelope.points().last() {
        //                 Some(p) if p.ticks <= ticks => color.clicked().alpha(0.7),
        //                 _ => return,
        //             },

        //         Clicked(Elem::EmptyRect, _, _) => color.clicked().alpha(0.7),
        //         Clicked(Elem::Point(_), _, _) => color.clicked(),

        //         _ => return,
        //     };

        //     let (y_bottom, y_top) = match value {
        //         ToggleValue(true) => (y + h / 4.0, rect.top()),
        //         ToggleValue(false) => (rect.bottom(), y - h / 4.0),
        //     };
        //     let start = [x, y_bottom];
        //     let end = [x, y_top];
        //     const THICKNESS: Scalar = 2.0;
        //     let line_idx = state.phantom_line_idx.get(&mut ui);
        //     Line::abs(start, end)
        //         .depth(1.0) // Place beind circles but in front of rectangles.
        //         .graphics_for(idx)
        //         .parent(idx)
        //         .color(color)
        //         .thickness(THICKNESS)
        //         .set(line_idx, &mut ui);
        // };

        events
    }
}
