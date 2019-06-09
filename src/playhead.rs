//! A widget representing a snappable playhead over some given range.

use conrod::{self, widget};
use core::time;
use ruler;

/// A widget representing a snappable playhead over some given range.
#[derive(WidgetCommon)]
pub struct Playhead {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    ruler: ruler::Ruler,
    /// The x dimensional range of the visible area of the tracks.
    visible_tracks_x: conrod::Range,
    style: Style,
}

/// The unique playhead state to be cached within the `Ui` between updates.
pub struct State {
    ids: Ids,
}

widget_ids! {
    struct Ids { line }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "theme.border_color")]
    color: Option<conrod::Color>,
}

/// Events that may occur within the Playhead widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    Pressed,
    DraggedTo(time::Ticks),
    Released,
}

impl Playhead {
    /// Start building a new Playhead widget.
    pub fn new(ruler: ruler::Ruler, visible_tracks_x: conrod::Range) -> Self {
        Playhead {
            ruler: ruler,
            visible_tracks_x: visible_tracks_x,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
        }
    }
}

impl conrod::Widget for Playhead {
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

    /// Update the state of the Playhead.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use conrod::{Colorable, Positionable};

        let widget::UpdateArgs {
            id,
            rect,
            state,
            style,
            ui,
            ..
        } = args;
        let Playhead {
            ruler,
            visible_tracks_x,
            ..
        } = self;

        let mut events = Vec::new();

        // Check for widget events:
        // - Press to
        for widget_event in ui.widget_input(id).events() {
            use conrod::{event, input};

            match widget_event {
                // If the `Playhead` was pressed with the left mouse button, react with a `Pressed`
                // event.
                event::Widget::Press(press) => {
                    if let event::Button::Mouse(input::MouseButton::Left, _) = press.button {
                        events.push(Event::Pressed);
                    }
                }

                // If the left mouse button was released from the playhead, reacti with a
                // `Released` event.
                event::Widget::Release(release) => {
                    if let event::Button::Mouse(input::MouseButton::Left, _) = release.button {
                        events.push(Event::Released);
                    }
                }

                _ => (),
            }
        }

        // If the playhead was dragged with the left mouse button, emit a drag event.
        if let Some(mouse) = ui.widget_input(id).mouse() {
            if mouse.buttons.left().is_down() {
                let mouse_abs_x = mouse.abs_xy()[0];
                let new_x = visible_tracks_x.clamp_value(mouse_abs_x);
                // Only react if the new position is different to the current position.
                if (rect.x() - new_x).abs() > 0.5 {
                    let x_offset = new_x - visible_tracks_x.start;
                    let target_position = time::Ticks((ruler.ticks_per_width() * x_offset) as _);
                    events.push(Event::DraggedTo(target_position));
                }
            }
        }

        // Instantiate the Line widget as the graphic representation of the playhead.
        let color = style.color(&ui.theme);
        let (color, thickness) = match ui.widget_input(id).mouse() {
            Some(mouse) => match mouse.buttons.left().is_down() {
                false => (color.highlighted(), 3.0),
                true => (color.clicked(), 3.0),
            },
            None => (color, 1.0),
        };
        let start = [0.0, 0.0];
        let end = [0.0, rect.h()];
        widget::Line::centred(start, end)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .thickness(thickness)
            .set(state.ids.line, ui);

        events
    }
}

impl conrod::Colorable for Playhead {
    builder_method!(color { style.color = Some(conrod::Color) });
}
