use super::{Bang, Point, Toggle};

/// A dynamically dispatched iterator yielding references to `T`.
pub type Points<'a, T> = Box<dyn Iterator<Item = &'a Point<T>>>;

/// A type representing a series of points of some kind supported by Jen's automation.
pub enum Dynamic<'a> {
    Bang(Points<'a, Bang>),
    Toggle(Points<'a, Toggle>),
    I8(Points<'a, i8>),
    I16(Points<'a, i16>),
    I32(Points<'a, i32>),
    I64(Points<'a, i64>),
    U8(Points<'a, u8>),
    U16(Points<'a, u16>),
    U32(Points<'a, u32>),
    U64(Points<'a, u64>),
    F32(Points<'a, f32>),
    F64(Points<'a, f64>),
}

macro_rules! impl_from_points_for_dynamic {
    ($($T:ident $variant:ident),* $(,)*) => {
        $(
            impl<'a> From<Points<'a, $T>> for Dynamic<'a> {
                fn from(points: Points<'a, $T>) -> Self {
                    Dynamic::$variant(points)
                }
            }
        )*
    }
}

impl_from_points_for_dynamic! {
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
