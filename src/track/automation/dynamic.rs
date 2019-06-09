use conrod::{self, widget};
use core;
use core::time::Ticks;
use num::{self, NumCast};
use track;

pub use core::automation::envelope::bounded::Dynamic as DynamicEnvelope;
pub use core::automation::envelope::bounded::Envelope;
pub use core::automation::envelope::Trait as EnvelopeTrait;
pub use core::automation::envelope::{Number, Point, ValueKind};
pub use core::automation::envelope::PointTrait;
pub use core::automation::bang::Bang;
pub use core::automation::toggle::Toggle;


/// A widget used for viewing and manipulating a series of points over time.
#[derive(WidgetCommon)]
pub struct Dynamic<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    bars: &'a [core::Bar],
    envelope: &'a DynamicEnvelope,
    /// The position of the playhead in ticks along with the change in position.
    pub maybe_playhead: Option<(Ticks, Ticks)>,
    style: Style,
}


/// The owned state to be cached within the `Ui`'s `Graph` between updates.
pub struct State {
    ids: Ids,
}

widget_ids! {
    struct Ids {
        numeric,
        toggle,
        bang,
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
    /// Some `Numeric` automation event.
    Numeric(super::numeric::Event<Number>),
    /// Some `Bang` automation event.
    Bang(super::bang::Event),
    /// Some `Toggle` automation event.
    Toggle(super::toggle::Event),
}

impl<'a> Dynamic<'a> {

    /// Construct a new default Dynamic.
    pub fn new(bars: &'a [core::Bar], envelope: &'a DynamicEnvelope) -> Self {
        Dynamic {
            bars: bars,
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

impl<'a> track::Widget for Dynamic<'a> {
    fn playhead(mut self, playhead: (Ticks, Ticks)) -> Self {
        self.maybe_playhead = Some(playhead);
        self
    }
}


impl<'a> conrod::Widget for Dynamic<'a> {
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
        ui.theme.widget_style::<Style>()
            .and_then(|default| default.common.maybe_y_dimension)
            .unwrap_or(conrod::position::Dimension::Absolute(super::super::DEFAULT_HEIGHT))
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use conrod::{Colorable, Positionable, Sizeable};

        // A function for instantiating a `automation::Numeric` widget.
        fn numeric_automation<T>(automation: Dynamic,
                                 args: widget::UpdateArgs<Dynamic>,
                                 env: &Envelope<T>) -> Vec<Event>
            where T: NumCast + Copy + Into<Number> + core::envelope::interpolation::Spatial + PartialEq + PartialOrd,
                  T::Scalar: num::Float,
                  Point<T>: PointTrait<X=Ticks, Y=T>,
        {
            use conrod::{Colorable, Positionable, Sizeable};
            let widget::UpdateArgs { id, state, style, ui, .. } = args;
            let Dynamic { maybe_playhead, bars, .. } = automation;

            let color = style.color(ui.theme());
            let point_radius = style.point_radius(ui.theme());

            super::numeric::Numeric::new(bars, env)
                .and_mut(|numeric| numeric.maybe_playhead = maybe_playhead)
                .middle_of(id)
                .wh_of(id)
                .color(color)
                .point_radius(point_radius)
                .set(state.ids.numeric, ui)
                .into_iter()
                .map(|event| match event {
                    super::numeric::Event::Interpolate(value) => {
                        let event = super::numeric::Event::Interpolate(value.into());
                        Event::Numeric(event)
                    },
                    super::numeric::Event::Mutate(mutate) => match mutate {
                        super::Mutate::DragPoint(drag_point) => {
                            let super::DragPoint { idx, ticks, value } = drag_point;
                            let drag_point = super::DragPoint {
                                idx: idx,
                                ticks: ticks,
                                value: value.into(),
                            };
                            let event = super::numeric::Event::Mutate(drag_point.into());
                            Event::Numeric(event)
                        },
                        super::Mutate::AddPoint(add_point) => {
                            let super::AddPoint { point: Point { ticks, value } } = add_point;
                            let point = Point {
                                ticks: ticks,
                                value: value.into(),
                            };
                            let add_point = super::AddPoint { point: point };
                            let event = super::numeric::Event::Mutate(add_point.into());
                            Event::Numeric(event)
                        },
                        super::Mutate::RemovePoint(remove_point) => {
                            let event: super::numeric::Event<Number> =
                                super::numeric::Event::Mutate(remove_point.into());
                            Event::Numeric(event)
                        },
                    }
                })
                .collect()
        }

        match *self.envelope {

            // Numeric envelopes.
            DynamicEnvelope::I8(ref env)  => numeric_automation(self, args, env),
            DynamicEnvelope::I16(ref env) => numeric_automation(self, args, env),
            DynamicEnvelope::I32(ref env) => numeric_automation(self, args, env),
            DynamicEnvelope::I64(ref env) => numeric_automation(self, args, env),
            DynamicEnvelope::U8(ref env)  => numeric_automation(self, args, env),
            DynamicEnvelope::U16(ref env) => numeric_automation(self, args, env),
            DynamicEnvelope::U32(ref env) => numeric_automation(self, args, env),
            DynamicEnvelope::U64(ref env) => numeric_automation(self, args, env),
            DynamicEnvelope::F32(ref env) => numeric_automation(self, args, env),
            DynamicEnvelope::F64(ref env) => numeric_automation(self, args, env),

            // Toggle envelopes.
            DynamicEnvelope::Toggle(ref env) => {
                let widget::UpdateArgs { id, state, style, ui, .. } = args;
                let Dynamic { bars, maybe_playhead, .. } = self;

                let color = style.color(&ui.theme);
                let point_radius = style.point_radius(&ui.theme);

                super::toggle::Toggle::new(bars, env)
                    .and_mut(|toggle| toggle.maybe_playhead = maybe_playhead)
                    .middle_of(id)
                    .wh_of(id)
                    .color(color)
                    .point_radius(point_radius)
                    .set(state.ids.toggle, ui)
                    .into_iter()
                    .map(|event| Event::Toggle(event))
                    .collect()
            },

            // Bang envelope.
            DynamicEnvelope::Bang(ref env) => {
                let widget::UpdateArgs { id, state, style, ui, .. } = args;
                let Dynamic { bars, maybe_playhead, .. } = self;

                let color = style.color(&ui.theme);
                let point_radius = style.point_radius(&ui.theme);

                super::bang::Bang::new(bars, env)
                    .and_mut(|bang| bang.maybe_playhead = maybe_playhead)
                    .middle_of(id)
                    .wh_of(id)
                    .color(color)
                    .point_radius(point_radius)
                    .set(state.ids.bang, ui)
                    .into_iter()
                    .map(|event| Event::Bang(event))
                    .collect()
            },

        }

    }

}

impl<'a> conrod::Colorable for Dynamic<'a> {
    builder_method!(color { style.color = Some(conrod::Color) });
}
