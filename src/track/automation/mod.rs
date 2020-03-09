use conrod_core::{self as conrod, Scalar};
use num::NumCast;
use time_calc::Ticks;

pub use env::bounded::Dynamic as DynamicEnvelope;
pub use env::bounded::Envelope;
pub use env::{Number, Point, Spatial, ValueKind};
pub use env::{PointTrait, Trait as EnvelopeTrait};

pub use self::bang::{Bang, BangValue};
pub use self::dynamic::Dynamic;
pub use self::numeric::Numeric;
pub use self::toggle::{Toggle, ToggleValue};

pub mod bang;
pub mod dynamic;
pub mod numeric;
pub mod toggle;

/// The different interactive elements of the automation widget.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Elem {
    /// Some place on a widget with an empty envelope
    EmptyRect,
    /// The space between two points.
    BetweenPoints(usize, usize),
    /// Some point in the automation envelope.
    Point(usize),
}

/// An event used to drag the point at the given index to the target ticks and value.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DragPoint<T> {
    /// The index into the envelope for the point which is to be dragged.
    pub idx: usize,
    /// The target ticks to which the point will be dragged.
    pub ticks: Ticks,
    /// The target value to which the point will be dragged.
    pub value: T,
}

impl<T> DragPoint<T> {
    pub fn apply(self, env: &mut Envelope<T>)
    where
        T: Copy,
    {
        let mut ticks = self.ticks;

        // Clamp dragging to the previous point.
        if let Some(min) = env.env.points.get(self.idx - 1).map(|p| p.ticks) {
            ticks = ::std::cmp::max(ticks, min);
        }

        // Clamp dragging to the next point.
        if let Some(max) = env.env.points.get(self.idx + 1).map(|p| p.ticks) {
            ticks = ::std::cmp::min(ticks, max);
        }

        if let Some(point) = env.env.points.get_mut(self.idx) {
            point.ticks = ticks;
            point.value = self.value;
        }
    }
}

/// An event used to add a new point to an Automation's envelope.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AddPoint<T> {
    /// The point which is to be added.
    pub point: Point<T>,
}

impl<T> AddPoint<T> {
    pub fn apply(self, env: &mut Envelope<T>)
    where
        T: Copy + PartialEq + Spatial,
        Point<T>: PointTrait<X = Ticks, Y = T>,
    {
        let num_points = env.env.points.len();
        let insert_idx = match env.point_idx_before(self.point.ticks) {
            Some(idx_before) if idx_before <= num_points => idx_before + 1,
            _ => 0,
        };
        env.env.points.insert(insert_idx, self.point);
    }
}

/// An event used to remove a point from an Automation's envelope.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RemovePoint {
    /// The index of the point which is to be removed.
    idx: usize,
}

impl RemovePoint {
    pub fn apply<T>(self, env: &mut Envelope<T>) -> Option<Point<T>> {
        if env.env.points.get(self.idx).is_some() {
            Some(env.env.points.remove(self.idx))
        } else {
            None
        }
    }
}

/// Events that when applied to the envelope cause some form of mutation.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Mutate<T> {
    /// Some envelope point was dragged.
    DragPoint(DragPoint<T>),
    /// Some envelope point was added.
    AddPoint(AddPoint<T>),
    /// Some envelope point was remove.
    RemovePoint(RemovePoint),
}

impl<T> Mutate<T> {
    /// Apply the mutation to the given `Envelope`.
    pub fn apply(self, envelope: &mut Envelope<T>)
    where
        T: Copy + PartialEq + Spatial,
        Point<T>: PointTrait<X = Ticks, Y = T>,
    {
        match self {
            Mutate::DragPoint(drag_point) => {
                drag_point.apply(envelope);
            }
            Mutate::AddPoint(add_point) => {
                add_point.apply(envelope);
            }
            Mutate::RemovePoint(remove_point) => {
                remove_point.apply(envelope);
            }
        }
    }
}

macro_rules! impl_from_for_mutation {
    ($t:ty, $variant:ident) => {
        impl<T> From<$t> for Mutate<T> {
            #[inline]
            fn from(mutation: $t) -> Self {
                Mutate::$variant(mutation)
            }
        }
    };
}

impl_from_for_mutation!(DragPoint<T>, DragPoint);
impl_from_for_mutation!(AddPoint<T>, AddPoint);
impl_from_for_mutation!(RemovePoint, RemovePoint);

// Determines the points that lie on either side of the playhead movement.
fn maybe_surrounding_elems<T>(
    total: Ticks,
    env: &Envelope<T>,
    start: Ticks,
    end: Ticks,
) -> Option<(Elem, Elem)>
where
    Point<T>: PointTrait<X = Ticks, Y = T>,
    T: PartialEq + Spatial,
{
    elem_at_ticks(total, env, start)
        .and_then(|start| elem_at_ticks(total, env, end).map(|end| (start, end)))
}

// If there is some envelope element at the given time in `ticks`, return it.
fn elem_at_ticks<T>(total: Ticks, env: &Envelope<T>, x: Ticks) -> Option<Elem>
where
    Point<T>: PointTrait<X = Ticks, Y = T>,
    T: PartialEq + Spatial,
{
    if Ticks(0) < x && x < total {
        env.point_at_with_idx(x)
            .map(|(i, _)| Elem::Point(i))
            .or_else(|| {
                env.point_idx_before(x).and_then(|start| {
                    let end = start + 1;
                    env.env
                        .points
                        .get(end)
                        .map(|_| Elem::BetweenPoints(start, end))
                })
            })
            .or(Some(Elem::EmptyRect))
    } else {
        None
    }
}

// Alter the given color depending upon its playhead.
fn color_elem_by_playhead(
    elem: Elem,
    playhead_delta_range: Option<(Elem, Elem)>,
    color: conrod::Color,
) -> conrod::Color {
    match is_elem_in_range(playhead_delta_range, elem) {
        true => color.clicked(),
        false => color,
    }
}

/// Converts the given value from its range to a y offset relative to the center of the height.
fn y_offset_from_value<T>(value: T, min: T, max: T, height: Scalar) -> Scalar
where
    T: NumCast,
{
    let value: Scalar = NumCast::from(value).expect("Can not cast to Scalar");
    let min: Scalar = NumCast::from(min).expect("Can not cast to Scalar");
    let max: Scalar = NumCast::from(max).expect("Can not cast to Scalar");
    let total_range = max - min;
    if total_range == 0.0 {
        min
    } else {
        let value_from_start = value - min;
        (value_from_start / total_range) * height - height / 2.0
    }
}

/// Indicates whether or not the given Envelope element is within the playhead's movement range.
fn is_elem_in_range(playhead_range: Option<(Elem, Elem)>, elem: Elem) -> bool {
    match playhead_range {
        None => false,
        Some((start_elem, end_elem)) => {
            let start = match start_elem {
                Elem::Point(start) => start,
                Elem::BetweenPoints(start, _) => start,
                // TODO: This should be fixed to consider the Rect area properly.
                _ => return false,
            };
            let end = match end_elem {
                Elem::Point(end) => end,
                Elem::BetweenPoints(_, end) => end,
                // TODO: This should be fixed to consider the Rect area properly.
                _ => return false,
            };
            match elem {
                Elem::Point(i) => i >= start && i <= end,
                Elem::BetweenPoints(a, b) => a >= start && b <= end,
                // TODO: This should be fixed to consider the Rect area properly.
                _ => return false,
            }
        }
    }
}
