//! A small suite of utility functions used throughout the crate.

/// Clamp a value between some range.
pub fn clamp<T>(n: T, start: T, end: T) -> T
where
    T: PartialOrd,
{
    let (min, max) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    if n < min {
        min
    } else if n > max {
        max
    } else {
        n
    }
}

/// Map a value from a given range to a new given range.
pub fn map_range<A, B>(val: A, in_min: A, in_max: A, out_min: B, out_max: B) -> B
where
    A: Into<f64>,
    B: Into<f64> + From<f64>,
{
    let val_f: f64 = val.into();
    let in_min_f: f64 = in_min.into();
    let in_max_f: f64 = in_max.into();
    let out_min_f: f64 = out_min.into();
    let out_max_f: f64 = out_max.into();
    ((val_f - in_min_f) / (in_max_f - in_min_f) * (out_max_f - out_min_f) + out_min_f).into()
}
