use super::bang::Bang;
use super::toggle::Toggle;
use super::{Number, ValueKind};
use super::{Point, Trait};
use std;
use time_calc as time;

/// An envelope with some min and max for the value range.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Envelope<T> {
    pub min: T,
    pub max: T,
    pub env: super::Envelope<T>,
}

impl<T> Envelope<T> {
    /// Construct a new, empty, default Envelope.
    pub fn new(min: T, max: T) -> Envelope<T> {
        Envelope::from_points(::std::iter::empty(), min, max)
    }

    /// Construct a new Envelope from the given min, max and points.
    pub fn from_points<I>(points: I, min: T, max: T) -> Envelope<T>
    where
        I: IntoIterator<Item = Point<T>>,
    {
        Envelope {
            min: min,
            max: max,
            env: points.into_iter().collect(),
        }
    }
}

impl<T> super::Envelope<T> {
    /// Convert the unbounded envelope to a bounded Envelope with the given `min` and `max`.
    pub fn bounded(self, min: T, max: T) -> Envelope<T> {
        Envelope {
            min: min,
            max: max,
            env: self,
        }
    }
}

impl<'a, T: 'a> Trait<'a> for Envelope<T>
where
    T: PartialEq + super::Spatial,
    Point<T>: super::PointTrait<X = time::Ticks, Y = T>,
{
    type X = time::Ticks;
    type Y = T;
    type Point = Point<T>;
    type Points = std::slice::Iter<'a, Point<T>>;
    fn points(&'a self) -> Self::Points {
        self.env.points.iter()
    }
}

/// A wrapper around the various kinds of bounded automation envelopes.
#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Bang(Envelope<Bang>),
    Toggle(Envelope<Toggle>),
    I8(Envelope<i8>),
    I16(Envelope<i16>),
    I32(Envelope<i32>),
    I64(Envelope<i64>),
    U8(Envelope<u8>),
    U16(Envelope<u16>),
    U32(Envelope<u32>),
    U64(Envelope<u64>),
    F32(Envelope<f32>),
    F64(Envelope<f64>),
}

impl Dynamic {
    /// Return the parameter value for the automation envelope at the given time in ticks.
    pub fn value_at_ticks(&self, x: time::Ticks) -> ValueKind {
        fn expect<T>(t: Option<T>) -> T {
            t.expect("Given `x` was out of range")
        }
        match *self {
            Dynamic::Bang(ref env) => ValueKind::Bang(env.closest_point(x).map(|p| p.ticks - x)),
            Dynamic::Toggle(ref env) => ValueKind::Toggle(*expect(env.y(x))),
            Dynamic::I8(ref env) => ValueKind::Number(Number::I8(expect(env.y(x)))),
            Dynamic::I16(ref env) => ValueKind::Number(Number::I16(expect(env.y(x)))),
            Dynamic::I32(ref env) => ValueKind::Number(Number::I32(expect(env.y(x)))),
            Dynamic::I64(ref env) => ValueKind::Number(Number::I64(expect(env.y(x)))),
            Dynamic::U8(ref env) => ValueKind::Number(Number::U8(expect(env.y(x)))),
            Dynamic::U16(ref env) => ValueKind::Number(Number::U16(expect(env.y(x)))),
            Dynamic::U32(ref env) => ValueKind::Number(Number::U32(expect(env.y(x)))),
            Dynamic::U64(ref env) => ValueKind::Number(Number::U64(expect(env.y(x)))),
            Dynamic::F32(ref env) => ValueKind::Number(Number::F32(expect(env.y(x)))),
            Dynamic::F64(ref env) => ValueKind::Number(Number::F64(expect(env.y(x)))),
        }
    }
}

macro_rules! impl_from_envelope_for_dynamic {
    ($($T:ident $variant:ident),* $(,)*) => {
        $(
            impl From<Envelope<$T>> for Dynamic {
                fn from(env: Envelope<$T>) -> Self {
                    Dynamic::$variant(env)
                }
            }
        )*
    };
}

impl_from_envelope_for_dynamic! {
    Bang Bang,
    Toggle Toggle,
    i8 I8,
    i16 I16,
    i32 I32,
    i64 I64,
    u8 U8,
    u16 U16,
    u32 U32,
    u64 U64,
    f32 F32,
    f64 F64,
}

impl Dynamic {
    /// Construct a bounded dynamic envelope from the given typed bounded envelope.
    pub fn from_envelope<T>(env: Envelope<T>) -> Dynamic
    where
        Self: From<Envelope<T>>,
    {
        Dynamic::from(env)
    }

    /// Construct a bounded dynamic envelope  from the given points.
    pub fn from_points<I, T>(points: I, min: T, max: T) -> Self
    where
        I: IntoIterator<Item = Point<T>>,
        super::Envelope<T>: std::iter::FromIterator<Point<T>>,
        Self: From<Envelope<T>>,
    {
        let envelope = Envelope::from_points(points, min, max);
        Dynamic::from_envelope(envelope)
    }
}
