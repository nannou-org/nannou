//! A mathematical foundation for nannou including point and vector types and a range of
//! helper/utility functions.

pub extern crate approx;
pub extern crate cgmath;
pub use self::cgmath::num_traits::{Float, NumCast, One};
pub use self::cgmath::*;
pub use geom::{pt2, pt3, pt4, vec2, vec3, vec4, Point2, Point3, Point4, Vector2, Vector3, Vector4};
use std::ops::Add;

/// Map a value from a given range to a new given range.
pub fn map_range<X, Y>(val: X, in_min: X, in_max: X, out_min: Y, out_max: Y) -> Y
where
    X: NumCast,
    Y: NumCast,
{
    macro_rules! unwrap_or_panic {
        ($result:expr, $arg:expr) => {
            $result.unwrap_or_else(|| panic!("[map_range] failed to cast {} arg to `f64`"))
        };
    }

    let val_f: f64 = unwrap_or_panic!(NumCast::from(val), "first");
    let in_min_f: f64 = unwrap_or_panic!(NumCast::from(in_min), "second");
    let in_max_f: f64 = unwrap_or_panic!(NumCast::from(in_max), "third");
    let out_min_f: f64 = unwrap_or_panic!(NumCast::from(out_min), "fourth");
    let out_max_f: f64 = unwrap_or_panic!(NumCast::from(out_max), "fifth");

    NumCast::from((val_f - in_min_f) / (in_max_f - in_min_f) * (out_max_f - out_min_f) + out_min_f)
        .unwrap_or_else(|| panic!("[map_range] failed to cast result to target type"))
}

/// The max between two partially ordered values.
pub fn partial_max<T>(a: T, b: T) -> T
where
    T: PartialOrd,
{
    if a >= b {
        a
    } else {
        b
    }
}

/// The min between two partially ordered values.
pub fn partial_min<T>(a: T, b: T) -> T
where
    T: PartialOrd,
{
    if a <= b {
        a
    } else {
        b
    }
}

/// Clamp a value between some range.
pub fn clamp<T>(n: T, start: T, end: T) -> T
where
    T: PartialOrd,
{
    if start <= end {
        if n < start {
            start
        } else if n > end {
            end
        } else {
            n
        }
    } else {
        if n < end {
            end
        } else if n > start {
            start
        } else {
            n
        }
    }
}

pub fn two<S>() -> S
where
    S: Add<Output = S> + One,
{
    S::one() + S::one()
}

/// Models the C++ fmod function.
#[inline]
pub fn fmod<F>(numer: F, denom: F) -> F
where
    F: Float,
{
    let rquot: F = (numer / denom).floor();
    numer - rquot * denom
}

/// Convert the given angle in degrees to the same angle in radians.
pub fn deg_to_rad<S>(deg: S) -> S
where
    S: BaseFloat,
{
    Rad::from(Deg(deg)).0
}

/// Convert the given angle in radians to the same angle in degrees.
pub fn rad_to_deg<S>(rad: S) -> S
where
    S: BaseFloat,
{
    Deg::from(Rad(rad)).0
}

/// Convert the given value as a number of "turns" into the equivalent angle in radians.
pub fn turns_to_rad<S>(turns: S) -> S
where
    S: BaseFloat,
{
    turns * NumCast::from(2.0 * ::std::f64::consts::PI).unwrap()
}

/// Convert the given value in radians to the equivalent value as a number of turns.
pub fn rad_to_turns<S>(rad: S) -> S
where
    S: BaseFloat,
{
    rad / NumCast::from(2.0 * ::std::f64::consts::PI).unwrap()
}
