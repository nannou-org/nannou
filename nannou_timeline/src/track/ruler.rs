use bars_duration_ticks;
use conrod_core::{self as conrod, widget, Colorable, Positionable};
use ruler;
use std;
use time_calc as time;
use track;

/// A track used for drawing the
#[derive(WidgetCommon)]
pub struct Ruler<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    ruler: ruler::Ruler,
    bars: &'a [time::TimeSig],
    ppqn: time::Ppqn,
    style: Style,
}

/// The unique `State` for the `Ruler` track.
pub struct State {
    ids: Ids,
}

widget_ids! {
    struct Ids {
        lines[],
        texts[],
        base_line,
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "theme.shape_color")]
    pub color: Option<conrod::Color>,
}

/// An event triggered whenever the user presses or drags the left mouse button across the ruler.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TicksTriggered {
    pub ticks: time::Ticks,
}

impl<'a> Ruler<'a> {
    /// Construct a new default Ruler track.
    pub fn new(ruler: ruler::Ruler, bars: &'a [time::TimeSig], ppqn: time::Ppqn) -> Self {
        Ruler {
            ruler: ruler,
            bars: bars,
            ppqn: ppqn,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
        }
    }
}

impl<'a> track::Widget for Ruler<'a> {}

impl<'a> conrod::Colorable for Ruler<'a> {
    builder_method!(color { style.color = Some(conrod::Color) });
}

impl<'a> conrod::Widget for Ruler<'a> {
    type State = State;
    type Style = Style;
    type Event = Option<TicksTriggered>;

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
            .unwrap_or(conrod::position::Dimension::Absolute(70.0))
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            rect,
            state,
            style,
            ui,
            ..
        } = args;
        let Ruler {
            ruler, bars, ppqn, ..
        } = self;

        // All that remains is to instantiate the graphics widgets.
        //
        // Check whether or not we need to do so by checking whether or not we're visible.
        if conrod::graph::algo::cropped_area_of_widget(ui.widget_graph(), id).is_none() {
            return None;
        }

        let (x, y, w, h) = rect.x_y_w_h();
        let line_color = style.color(ui.theme());
        let half_w = w / 2.0;
        let total_ticks = bars_duration_ticks(bars.iter().cloned(), ppqn);

        // `TicksTriggered` occurs while the ruler is pressed by the left mouse button.
        let mut ticks_triggered = None;
        if let Some(mouse) = ui.widget_input(id).mouse() {
            if mouse.buttons.left().is_down() {
                let abs_x = mouse.abs_xy()[0];
                let x = abs_x - rect.left();
                let ticks = time::Ticks(
                    ((x / w) * total_ticks.ticks() as conrod::Scalar) as time::calc::Ticks,
                );
                let ticks = std::cmp::max(std::cmp::min(ticks, total_ticks), time::Ticks(0));
                ticks_triggered = Some(TicksTriggered { ticks });
            }
        }

        // Instantiate the base Line.
        let start = [x - half_w, y];
        let end = [x + half_w, y];
        widget::Line::abs(start, end)
            .color(line_color)
            .graphics_for(id)
            .parent(id)
            .set(state.ids.base_line, ui);

        let markers_in_ticks = ruler.markers_in_ticks(bars.iter().cloned(), ppqn);

        // Find the number of text and line widgets that we'll need.
        let mut num_texts = 0;
        let mut num_lines = 0;
        for bar_markers in markers_in_ticks.clone() {
            num_texts += 1;
            for _ in bar_markers {
                num_lines += 1;
            }
        }

        // Check we have enough text indices.
        if state.ids.texts.len() < num_texts {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.texts.resize(num_texts, id_gen));
        }

        // Check we have enough line indices.
        if state.ids.lines.len() < num_lines {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.lines.resize(num_lines, id_gen));
        }

        let markers_in_width_steps = markers_in_ticks
            .map(|markers| markers.map(|ticks| ticks.beats(ppqn) * ruler.width_per_beat));
        let markers_in_ruler_divs = ruler.markers_in_divisions(bars.iter().cloned(), ppqn);
        let width_steps_with_divisions = markers_in_width_steps.zip(markers_in_ruler_divs);
        let mut line_ids = state.ids.lines.iter();
        let iter = width_steps_with_divisions
            .enumerate()
            .zip(state.ids.texts.iter());
        for ((i, (bar_width_steps, bar_divs)), &text_id) in iter {
            let bar_width_steps_and_divs = bar_width_steps.zip(bar_divs);
            for (j, (width_step, div)) in bar_width_steps_and_divs.enumerate() {
                let &line_id = line_ids.next().unwrap();

                // The height of the marker will be determined by the simplest div that
                // could represent this step in ticks.
                let marker_height_weight = match div {
                    time::Division::Bar => 1.0,
                    time::Division::Beat => 0.5,
                    time::Division::Quaver => 0.25,
                    _ => 0.125,
                };
                let marker_height = marker_height_weight * h;
                let half_marker_height = marker_height / 2.0;
                let line_x = width_step - half_w;
                let start = [x + line_x, y - half_marker_height];
                let end = [x + line_x, y + half_marker_height];
                widget::Line::abs(start, end)
                    .color(line_color)
                    .graphics_for(id)
                    .parent(id)
                    .set(line_id, ui);

                // If we're doing the marker for a Bar, we'll add some text to it too.
                if j == 0 {
                    // Construct the text form.
                    let nth_bar = i + 1;
                    let string = nth_bar.to_string();
                    const MAX_FONT_SIZE: conrod::FontSize = 12;
                    let font_height = (half_marker_height / 2.0) / 2.0;
                    let font_size = std::cmp::max(font_height as conrod::FontSize, MAX_FONT_SIZE);
                    const TEXT_PADDING: f64 = 5.0;
                    widget::Text::new(&string)
                        .color(line_color)
                        .font_size(font_size)
                        .mid_top_with_margin_on(id, TEXT_PADDING)
                        .right(TEXT_PADDING)
                        .graphics_for(id)
                        .parent(id)
                        .set(text_id, ui);
                }
            }
        }

        ticks_triggered
    }
}
