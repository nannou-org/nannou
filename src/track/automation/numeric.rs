use conrod::{self, widget};
use core::{self, BarIterator};
use core::time::Ticks;
use num::{self, NumCast};
use ruler;
use track;

pub use core::automation::envelope::bounded::Envelope;
pub use core::automation::envelope::Trait as EnvelopeTrait;
pub use core::automation::envelope::Point;
pub use core::automation::envelope::PointTrait;


/// For viewing and manipulating series of numerically valued points over time.
#[derive(WidgetCommon)]
pub struct Numeric<'a, T: 'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    envelope: &'a Envelope<T>,
    bars: &'a [core::Bar],
    /// The position and change in position of the playhead in ticks, respectively.
    pub maybe_playhead: Option<(Ticks, Ticks)>,
    style: Style,
}

/// Unique state for the Numeric automation.
pub struct State {
    ids: Ids,
}

widget_ids! {
    struct Ids {
        circles[],
        lines[],
        phantom_left_line,
        phantom_right_line,
        phantom_circle,
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
pub enum Event<T> {
    /// Upon playhead movement, represents new numeric value at playhead.
    Interpolate(T),
    /// Some mutation to be applied to the envelope.
    Mutate(super::Mutate<T>),
}


impl<'a, T> Numeric<'a, T> {

    /// Construct a new default Automation.
    pub fn new(bars: &'a [core::Bar],
               envelope: &'a Envelope<T>) -> Self
    {
        Numeric {
            bars: bars,
            maybe_playhead: None,
            envelope: envelope,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
        }
    }

    builder_methods!{
        pub point_radius { style.point_radius = Some(conrod::Scalar) }
    }

}

impl<'a, T> track::Widget for Numeric<'a, T>
    where Numeric<'a, T>: conrod::Widget,
{
    fn playhead(mut self, playhead: (Ticks, Ticks)) -> Self {
        self.maybe_playhead = Some(playhead);
        self
    }
}


impl<'a, T> conrod::Colorable for Numeric<'a, T> {
    builder_method!(color { style.color = Some(conrod::Color) });
}


impl<'a, T> conrod::Widget for Numeric<'a, T>
    where T: NumCast + Copy + core::envelope::interpolation::Spatial + PartialEq + PartialOrd,
          T::Scalar: num::Float,
          Point<T>: PointTrait<X=Ticks, Y=T>,
{
    type State = State;
    type Style = Style;
    type Event = Vec<Event<T>>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_y_dimension(&self, ui: &conrod::Ui) -> conrod::position::Dimension {
        ui.theme.widget_style::<Style>()
            .and_then(|default| default.common.maybe_y_dimension)
            .unwrap_or(conrod::position::Dimension::Absolute(super::super::DEFAULT_HEIGHT))
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use conrod::{Colorable, Positionable};
        use conrod::utils::{clamp, map_range};
        use super::Elem;

        let widget::UpdateArgs { id, rect, state, style, ui, .. } = args;
        let Numeric { envelope, bars, maybe_playhead, .. } = self;

        let num_points = envelope.points().count();
        let num_lines = if num_points == 0 { 0 } else { num_points - 1 };

        // Ensure we have a circle index for each point.
        if state.ids.circles.len() < num_points {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.circles.resize(num_points, id_gen));
        }

        // Ensure we have a line index for each point.
        if state.ids.lines.len() < num_lines {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.lines.resize(num_lines, id_gen));
        }

        let (x, y, w, h) = rect.x_y_w_h();
        let color = style.color(&ui.theme);
        let point_radius = style.point_radius(&ui.theme);
        let total_ticks = bars.iter().cloned().total_duration();
        let min = envelope.min;
        let max = envelope.max;

        let rel_x_from_ticks = |ticks: Ticks| ruler::x_offset_from_ticks(ticks, total_ticks, w);
        let rel_y_from_value = |value: T| super::y_offset_from_value(value, min, max, h);

        let x_from_ticks = |ticks: Ticks| rel_x_from_ticks(ticks) + x;
        let y_from_value = |value: T| rel_y_from_value(value) + y;

        let ticks_from_x = |x: conrod::Scalar| Ticks(map_range(x, rect.left(), rect.right(), 0, total_ticks.ticks()));
        let value_from_y = |y: conrod::Scalar| map_range(y, rect.bottom(), rect.top(), min, max);

        let clamped_ticks_from_x = |x: conrod::Scalar| clamp(ticks_from_x(x), Ticks(0), total_ticks);
        let clamped_value_from_y = |y: conrod::Scalar| clamp(value_from_y(y), min, max);

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
            },
            _ => None,
        };

        let point_color = |i: usize,
                           color: conrod::Color,
                           ui: &conrod::Ui,
                           id: widget::Id| -> conrod::Color
        {
            let elem = Elem::Point(i);
            let color = super::color_elem_by_playhead(elem, playhead_delta_range, color);
            match ui.widget_input(id).mouse() {
                Some(mouse) => match mouse.buttons.left().is_down() {
                    false => color.highlighted(),
                    true => color.clicked(),
                },
                None => color,
            }
        };

        let line_color = |start: usize, end: usize,
                          color: conrod::Color,
                          ui: &conrod::Ui,
                          id: widget::Id| -> conrod::Color
        {
            let elem = Elem::BetweenPoints(start, end);
            let color = super::color_elem_by_playhead(elem, playhead_delta_range, color);
            match ui.widget_input(id).mouse() {
                Some(mouse) => match mouse.buttons.left().is_down() {
                    false => color.highlighted().alpha(0.8),
                    true => color.clicked().alpha(0.5),
                },
                None => color,
            }
        };

        let mut events = Vec::new();

        // A function for instantiating a Circle widget for a point.
        let point_widget = |i: usize,
                            point_id: widget::Id,
                            xy_offset: conrod::Point,
                            ui: &mut conrod::UiCell,
                            events: &mut Vec<Event<T>>|
        {
            for widget_event in ui.widget_input(point_id).events() {
                use conrod::{event, input};

                match widget_event {

                    // Check to see whether or not the point has been dragged.
                    event::Widget::Drag(drag) if drag.button == input::MouseButton::Left => {
                        let point_rect = ui.rect_of(point_id).unwrap();
                        let drag_to_abs_xy = conrod::utils::vec2_add(drag.to, point_rect.xy());
                        let drag_point = super::DragPoint {
                            idx: i,
                            ticks: clamped_ticks_from_x(drag_to_abs_xy[0]),
                            value: clamped_value_from_y(drag_to_abs_xy[1]),
                        };
                        events.push(Event::Mutate(drag_point.into()));
                    },

                    // Check to see whether or not a point has been deleted.
                    event::Widget::Click(click) if click.button == input::MouseButton::Right => {
                        let remove_point = super::RemovePoint { idx: i };
                        events.push(Event::Mutate(remove_point.into()));
                    },

                    _ => (),
                }
            }

            let point_color = point_color(i, color, ui, point_id);
            widget::Circle::fill(point_radius)
                .xy_relative_to(id, xy_offset)
                .parent(id)
                .color(point_color)
                .set(point_id, ui);
        };

        const LINE_THICKNESS: conrod::Scalar = 1.0;

        let mut iter = envelope.points().enumerate().zip(state.ids.circles.iter());

        // Do the first point manually so that we don't have to do any checks for the lines.
        if let Some(((i, &first_point), &first_point_id)) = iter.next() {
            let x_offset = rel_x_from_ticks(first_point.ticks);
            let y_offset = rel_y_from_value(first_point.value);
            let xy_offset = [x_offset, y_offset];
            point_widget(i, first_point_id, xy_offset, ui, &mut events);

            let mut prev_i = i;
            let mut prev_point_id = first_point_id;
            let mut prev_xy_offset = xy_offset;
            let mut line_ids = state.ids.lines.iter();
            let mut next = || iter.next().and_then(|p| line_ids.next().map(|l| (p, l)));

            // Now the remaining points along with the line to each.
            while let Some((((i, &point), &point_id), &line_id)) = next() {
                let x_offset = rel_x_from_ticks(point.ticks);
                let y_offset = rel_y_from_value(point.value);
                let xy_offset = [x_offset, y_offset];

                // Instantiate the line widget between points.
                let line_color = line_color(prev_i, i, color, &ui, line_id);
                let thickness = LINE_THICKNESS +
                    match ui.widget_input(prev_point_id).mouse()
                        .or_else(|| ui.widget_input(point_id).mouse())
                {
                    Some(_mouse) => 1.0,
                    None => 0.0,
                };

                let start = [x + prev_xy_offset[0], y + prev_xy_offset[1]];
                let end = [x + x_offset, y + y_offset];
                widget::Line::abs(start, end)
                    .depth(1.0) // Put the lines behind the points.
                    .graphics_for(id)
                    .parent(id)
                    .color(line_color)
                    .thickness(thickness)
                    .set(line_id, ui);

                // And now the next point's circle widget.
                point_widget(i, point_id, xy_offset, ui, &mut events);

                prev_i = i;
                prev_xy_offset = xy_offset;
                prev_point_id = point_id;
            }
        }

        // Check to see whether or not a new point should be added.
        for click in ui.widget_input(id).clicks().left() {
            let click_abs_xy = conrod::utils::vec2_add(click.xy, rect.xy());
            let point = core::automation::Point {
                ticks: clamped_ticks_from_x(click_abs_xy[0]),
                value: clamped_value_from_y(click_abs_xy[1]),
            };
            let add_point = super::AddPoint { point: point };
            events.push(Event::Mutate(add_point.into()));
        }

        // If the mouse is over the widget, draw a "phantom point" at the cursor.
        if let Some((mouse_abs_xy, is_left_down)) = ui.widget_input(id).mouse()
            .map(|m| (m.abs_xy(), m.buttons.left().is_down()))
        {
            let (phantom_color, line_thickness) = match is_left_down {
                true => (color.clicked().alpha(0.7), LINE_THICKNESS + 1.0),
                false => (color.highlighted().alpha(0.5), LINE_THICKNESS + 1.0),
            };

            // Snap to the value and then back to `x` to ensure we have a realistic point position.
            let ticks = ticks_from_x(mouse_abs_xy[0]);
            let value = value_from_y(mouse_abs_xy[1]);

            let left_idx = envelope.point_on_or_before_with_idx(ticks).map(|(i, _)| i);
            let right_idx = envelope.point_after_with_idx(ticks).map(|(i, _)| i);;

            let point_x = x_from_ticks(ticks);
            let point_y = y_from_value(value);
            let point_xy = [point_x, point_y];

            // Instantiate the Circle widget at the phantom cursor point.
            widget::Circle::fill(point_radius)
                .depth(0.5) // Put circle behind other points.
                .color(phantom_color)
                .xy(point_xy)
                .parent(id)
                .graphics_for(id)
                .set(state.ids.phantom_circle, ui);

            let phantom_line_widget = |point: Point<T>, line_id: widget::Id, ui: &mut conrod::UiCell| {
                let left_point_x = x_from_ticks(point.ticks);
                let left_point_y = y_from_value(point.value);
                let start = [left_point_x, left_point_y];
                widget::Line::abs(start, point_xy)
                    .depth(1.0) // Put the lines behind the points.
                    .graphics_for(id)
                    .parent(id)
                    .color(phantom_color)
                    .thickness(line_thickness)
                    .set(line_id, ui);
            };

            if let Some(&left_point) = left_idx.and_then(|p_idx| envelope.points().nth(p_idx)) {
                phantom_line_widget(left_point, state.ids.phantom_left_line, ui);
            }

            if let Some(&right_point) = right_idx.and_then(|p_idx| envelope.points().nth(p_idx)) {
                phantom_line_widget(right_point, state.ids.phantom_right_line, ui);
            }
        }

        events
    }

}
