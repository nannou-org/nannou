use std;
use time_calc as time;

pub use self::bang::Bang;
pub use self::point::Point;
pub use self::point::Trait as PointTrait;
pub use self::toggle::Toggle;
pub use envelope::interpolation::Spatial;
pub use envelope::Envelope as Trait;

pub mod bang;
pub mod bounded;
pub mod point;
pub mod points;
pub mod toggle;

/// A generic envelope type.
#[derive(Clone, Debug, PartialEq)]
pub struct Envelope<T> {
    pub points: Vec<Point<T>>,
}

/// A wrapper around the various kinds of automation envelopes.
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

/// A wrapper around the different kinds of automatable numeric types.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Number {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

/// A wrapper around the different kinds of automatable value types.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum ValueKind {
    /// The offset from the closest `Bang` in Ticks (if there is a `Bang`).
    Bang(Option<time::Ticks>),
    Toggle(bool),
    Number(Number),
}

impl<T> std::iter::FromIterator<Point<T>> for Envelope<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Point<T>>,
    {
        Envelope {
            points: iter.into_iter().collect(),
        }
    }
}

impl<T> From<Vec<Point<T>>> for Envelope<T> {
    fn from(points: Vec<Point<T>>) -> Self {
        Envelope { points: points }
    }
}

impl<'a, T: 'a> Trait<'a> for Envelope<T>
where
    T: PartialEq + Spatial,
    Point<T>: PointTrait<X = time::Ticks, Y = T>,
{
    type X = time::Ticks;
    type Y = T;
    type Point = Point<T>;
    type Points = std::slice::Iter<'a, Point<T>>;
    fn points(&'a self) -> Self::Points {
        self.points.iter()
    }
}

macro_rules! impl_from_for_number {
    ($num:ty, $variant:ident) => {
        impl From<$num> for Number {
            fn from(n: $num) -> Self {
                Number::$variant(n)
            }
        }
    };
}

impl_from_for_number!(i8, I8);
impl_from_for_number!(i16, I16);
impl_from_for_number!(i32, I32);
impl_from_for_number!(i64, I64);
impl_from_for_number!(u8, U8);
impl_from_for_number!(u16, U16);
impl_from_for_number!(u32, U32);
impl_from_for_number!(u64, U64);
impl_from_for_number!(f32, F32);
impl_from_for_number!(f64, F64);

macro_rules! impl_from_for_value_kind {
    ($value_ty:ty, $variant:ident) => {
        impl From<$value_ty> for ValueKind {
            fn from(v: $value_ty) -> Self {
                ValueKind::$variant(v)
            }
        }
    };
}

impl_from_for_value_kind!(bool, Toggle);
impl_from_for_value_kind!(Option<time::Ticks>, Bang);

impl<N> From<N> for ValueKind
where
    N: Into<Number>,
{
    fn from(n: N) -> Self {
        ValueKind::Number(n.into())
    }
}

macro_rules! impl_from_envelope_for_dynamic {
    ($value_type:ty, $variant:ident) => {
        impl From<Envelope<$value_type>> for Dynamic {
            fn from(env: Envelope<$value_type>) -> Self {
                Dynamic::$variant(env)
            }
        }
    };
}

impl_from_envelope_for_dynamic!(Bang, Bang);
impl_from_envelope_for_dynamic!(Toggle, Toggle);
impl_from_envelope_for_dynamic!(i8, I8);
impl_from_envelope_for_dynamic!(i16, I16);
impl_from_envelope_for_dynamic!(i32, I32);
impl_from_envelope_for_dynamic!(i64, I64);
impl_from_envelope_for_dynamic!(u8, U8);
impl_from_envelope_for_dynamic!(u16, U16);
impl_from_envelope_for_dynamic!(u32, U32);
impl_from_envelope_for_dynamic!(u64, U64);
impl_from_envelope_for_dynamic!(f32, F32);
impl_from_envelope_for_dynamic!(f64, F64);

impl<T> std::iter::FromIterator<Point<T>> for Dynamic
where
    Envelope<T>: Into<Dynamic>,
{
    fn from_iter<I>(points: I) -> Self
    where
        I: IntoIterator<Item = Point<T>>,
    {
        let env: Envelope<T> = points.into_iter().collect();
        env.into()
    }
}

/// A macro to simplify implementation of `expect_$type` methods on the ValueKind type.
macro_rules! fn_expect_num {
    ($method_name:ident, $return_type:ty, $variant:ident) => {
        /// Forces the specified number type from the `ValueKind`.
        ///
        /// **Panics** if the ValueKind variant was of a different type.
        pub fn $method_name(&self) -> $return_type {
            match *self {
                ValueKind::Number(Number::$variant(value)) => value,
                _ => panic!("`ValueKind` expected a {:?}", stringify!($method_name)),
            }
        }
    };
}

/// A macro to simplify implementation of casting methods on the ValueKind type.
macro_rules! fn_as_type {
    ($method_name:ident, $return_type:ty) => {
        /// Casts the value from the current variant to the specified type.
        ///
        /// The `Bang` variant will always be cast to 0.
        ///
        /// The `Toggle` variant will always be cast to u8 before being cast to the specified type.
        pub fn $method_name(&self) -> $return_type {
            match *self {
                ValueKind::Bang(_) => 0 as $return_type,
                ValueKind::Toggle(b) => b as u8 as $return_type,
                ValueKind::Number(Number::I8(n)) => n as $return_type,
                ValueKind::Number(Number::I16(n)) => n as $return_type,
                ValueKind::Number(Number::I32(n)) => n as $return_type,
                ValueKind::Number(Number::I64(n)) => n as $return_type,
                ValueKind::Number(Number::U8(n)) => n as $return_type,
                ValueKind::Number(Number::U16(n)) => n as $return_type,
                ValueKind::Number(Number::U32(n)) => n as $return_type,
                ValueKind::Number(Number::U64(n)) => n as $return_type,
                ValueKind::Number(Number::F32(n)) => n as $return_type,
                ValueKind::Number(Number::F64(n)) => n as $return_type,
            }
        }
    };
}

impl ValueKind {
    fn_expect_num!(expect_u8, u8, U8);
    fn_expect_num!(expect_u16, u16, U16);
    fn_expect_num!(expect_u32, u32, U32);
    fn_expect_num!(expect_u64, u64, U64);
    fn_expect_num!(expect_i8, i8, I8);
    fn_expect_num!(expect_i16, i16, I16);
    fn_expect_num!(expect_i32, i32, I32);
    fn_expect_num!(expect_i64, i64, I64);
    fn_expect_num!(expect_f32, f32, F32);
    fn_expect_num!(expect_f64, f64, F64);

    /// Forces the specified type from the `ValueKind`.
    ///
    /// **Panics** if the ValueKind variant was of a different type.
    pub fn expect_bool(&self) -> bool {
        match *self {
            ValueKind::Toggle(b) => b,
            _ => panic!("`ValueKind` expected a bool"),
        }
    }

    fn_as_type!(as_u8, u8);
    fn_as_type!(as_u16, u16);
    fn_as_type!(as_u32, u32);
    fn_as_type!(as_u64, u64);
    fn_as_type!(as_i8, i8);
    fn_as_type!(as_i16, i16);
    fn_as_type!(as_i32, i32);
    fn_as_type!(as_i64, i64);
    fn_as_type!(as_f32, f32);
    fn_as_type!(as_f64, f64);
}
