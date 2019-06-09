use super::bang::Bang;
use super::toggle::Toggle;
use time;

pub use envelope::Point as Trait;

/// An automation point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point<T> {
    pub ticks: time::Ticks,
    pub value: T,
}

impl<T> Point<T> {
    /// Constructor for a new Point.
    pub fn new(ticks: time::Ticks, value: T) -> Point<T> {
        Point {
            ticks: ticks,
            value: value,
        }
    }

    /// Map a `Point`'s value field from one type to another.
    pub fn map_value<F, U>(self, mut f: F) -> Point<U>
    where
        F: FnMut(T) -> U,
    {
        let Point { ticks, value } = self;
        Point {
            ticks: ticks,
            value: f(value),
        }
    }
}

/// Fast implementation of envelope::Point for most Point types.
macro_rules! impl_point_for {
    ($T: ident, $Scalar: ident) => {
        /// Implement envelope::Point for Points with floating point parameters.
        impl Trait for Point<$T> {
            type X = time::Ticks;
            type Y = $T;
            fn x_to_scalar(x: time::Ticks) -> $Scalar {
                x.ticks() as $Scalar
            }
            fn x(&self) -> time::Ticks {
                self.ticks
            }
            fn y(&self) -> $T {
                self.value
            }
        }
    };
}

impl_point_for!(i8, f32);
impl_point_for!(i16, f32);
impl_point_for!(i32, f32);
impl_point_for!(i64, f64);

impl_point_for!(u8, f32);
impl_point_for!(u16, f32);
impl_point_for!(u32, f32);
impl_point_for!(u64, f64);

impl_point_for!(f32, f32);
impl_point_for!(f64, f64);

/// A bang doesn't yet have a Point implementation, so create one.
impl ::envelope::Point for Point<Bang> {
    type X = time::Ticks;
    type Y = Bang;
    fn x_to_scalar(_: time::Ticks) -> f32 {
        0.0
    }
    fn x(&self) -> time::Ticks {
        self.ticks
    }
    fn y(&self) -> Bang {
        self.value
    }
    fn interpolate(_: time::Ticks, _: &Point<Bang>, _: &Point<Bang>) -> Bang {
        Bang
    }
}

/// A bool doesn't yet have a Point implementation, so create one.
impl ::envelope::Point for Point<Toggle> {
    type X = time::Ticks;
    type Y = Toggle;
    fn x_to_scalar(x: time::Ticks) -> f32 {
        x.ticks() as f32
    }
    fn x(&self) -> time::Ticks {
        self.ticks
    }
    fn y(&self) -> Toggle {
        self.value
    }
    fn interpolate(x: time::Ticks, start: &Point<Toggle>, end: &Point<Toggle>) -> Toggle {
        if x == end.ticks {
            end.value
        } else if x >= start.ticks {
            start.value
        } else {
            panic!(
                "Failed to interpolate toggle envelope - ticks {:?} out of range.",
                x
            );
        }
    }
}
