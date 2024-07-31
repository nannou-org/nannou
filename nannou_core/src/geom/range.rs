//! A type for working with one-dimensional ranges.

use core::ops::{Add, Neg, Sub};

use crate::geom::scalar;
use crate::math::num_traits::{Float, NumCast, One, Zero};
use crate::math::{self, two};

/// Some start and end position along a single axis.
///
/// As an example, a **Rect** is made up of two **Range**s; one along the *x* axis, and one along
/// the *y* axis.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Range<S = scalar::Default> {
    /// The start of some `Range` along an axis.
    pub start: S,
    /// The end of some `Range` along an axis.
    pub end: S,
}

/// Represents either the **Start** or **End** **Edge** of a **Range**.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Edge {
    /// The beginning of a **Range**.
    Start,
    /// The end of a **Range**.
    End,
}

/// Describes alignment along a range.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Align {
    Start,
    Middle,
    End,
}

impl<S> Range<S>
where
    S: Copy,
{
    /// Construct a new `Range` from a given range, i.e. `Range::new(start, end)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range { start: 0.0, end: 10.0 }, Range::new(0.0, 10.0));
    /// ```
    pub fn new(start: S, end: S) -> Self {
        Range { start, end }
    }

    /// Construct a new `Range` from a given length and its centered position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0), Range::from_pos_and_len(5.0, 10.0));
    /// assert_eq!(Range::new(-5.0, 1.0), Range::from_pos_and_len(-2.0, 6.0));
    /// assert_eq!(Range::new(-100.0, 200.0), Range::from_pos_and_len(50.0, 300.0));
    /// ```
    pub fn from_pos_and_len(pos: S, len: S) -> Self
    where
        S: Float,
    {
        let half_len = len / two();
        let start = pos - half_len;
        let end = pos + half_len;
        Range::new(start, end)
    }

    /// The `start` value subtracted from the `end` value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).magnitude(), 10.0);
    /// assert_eq!(Range::new(5.0, -5.0).magnitude(), -10.0);
    /// assert_eq!(Range::new(15.0, 10.0).magnitude(), -5.0);
    /// ```
    pub fn magnitude(&self) -> S
    where
        S: Sub<S, Output = S>,
    {
        self.end - self.start
    }

    /// The absolute length of the Range aka the absolute magnitude.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).len(), 10.0);
    /// assert_eq!(Range::new(5.0, -5.0).len(), 10.0);
    /// assert_eq!(Range::new(15.0, 10.0).len(), 5.0);
    /// ```
    pub fn len(&self) -> S
    where
        S: Neg<Output = S> + PartialOrd + Sub<S, Output = S> + Zero,
    {
        let mag = self.magnitude();
        let zero = S::zero();
        if mag < zero {
            -mag
        } else {
            mag
        }
    }

    /// Return the value directly between the start and end values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).middle(), 0.0);
    /// assert_eq!(Range::new(5.0, -5.0).middle(), 0.0);
    /// assert_eq!(Range::new(10.0, 15.0).middle(), 12.5);
    /// assert_eq!(Range::new(20.0, 40.0).middle(), 30.0);
    /// assert_eq!(Range::new(20.0, -40.0).middle(), -10.0);
    /// ```
    pub fn middle(&self) -> S
    where
        S: Float,
    {
        (self.end + self.start) / two()
    }

    /// The current range with its start and end values swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).invert(), Range::new(5.0, -5.0));
    /// assert_eq!(Range::new(-10.0, 10.0).invert(), Range::new(10.0, -10.0));
    /// assert_eq!(Range::new(0.0, 7.25).invert(), Range::new(7.25, 0.0));
    /// assert_eq!(Range::new(5.0, 1.0).invert(), Range::new(1.0, 5.0));
    /// ```
    pub fn invert(self) -> Self {
        Range {
            start: self.end,
            end: self.start,
        }
    }

    /// Map the given scalar from `Self` to some other given `Range`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(0.0, 5.0);
    ///
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.map_value(2.5, &b), 5.0);
    /// assert_eq!(a.map_value(0.0, &b), 0.0);
    /// assert_eq!(a.map_value(5.0, &b), 10.0);
    /// assert_eq!(a.map_value(-5.0, &b), -10.0);
    /// assert_eq!(a.map_value(10.0, &b), 20.0);
    ///
    /// let c = Range::new(10.0, -10.0);
    /// assert_eq!(a.map_value(2.5, &c), 0.0);
    /// assert_eq!(a.map_value(0.0, &c), 10.0);
    /// assert_eq!(a.map_value(5.0, &c), -10.0);
    /// assert_eq!(a.map_value(-5.0, &c), 30.0);
    /// assert_eq!(a.map_value(10.0, &c), -30.0);
    /// ```
    pub fn map_value(&self, value: S, other: &Self) -> S
    where
        S: NumCast,
    {
        math::map_range(value, self.start, self.end, other.start, other.end)
    }

    /// Interpolates the **Range** using the given `weight`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let r = Range::new(-5.0, 5.0);
    /// assert_eq!(r.lerp(0.0), -5.0);
    /// assert_eq!(r.lerp(1.0), 5.0);
    /// assert_eq!(r.lerp(0.5), 0.0);
    /// ```
    pub fn lerp(&self, amount: S) -> S
    where
        S: Float,
    {
        self.start + ((self.end - self.start) * amount)
    }

    /// Shift the `Range` start and end points by a given scalar.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).shift(5.0), Range::new(5.0, 10.0));
    /// assert_eq!(Range::new(0.0, 5.0).shift(-5.0), Range::new(-5.0, 0.0));
    /// assert_eq!(Range::new(5.0, -5.0).shift(-5.0), Range::new(0.0, -10.0));
    /// ```
    pub fn shift(self, amount: S) -> Self
    where
        S: Add<Output = S>,
    {
        Range {
            start: self.start + amount,
            end: self.end + amount,
        }
    }

    /// The direction of the Range represented as a normalised scalar.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).direction(), 1.0);
    /// assert_eq!(Range::new(0.0, 0.0).direction(), 0.0);
    /// assert_eq!(Range::new(0.0, -5.0).direction(), -1.0);
    /// ```
    pub fn direction(&self) -> S
    where
        S: Neg<Output = S> + One + PartialOrd + Zero,
    {
        if self.start < self.end {
            S::one()
        } else if self.start > self.end {
            -S::one()
        } else {
            S::zero()
        }
    }

    /// Converts the Range to an absolute Range by ensuring that `start` <= `end`.
    ///
    /// If `start` > `end`, then the start and end points will be swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).absolute(), Range::new(0.0, 5.0));
    /// assert_eq!(Range::new(5.0, 1.0).absolute(), Range::new(1.0, 5.0));
    /// assert_eq!(Range::new(10.0, -10.0).absolute(), Range::new(-10.0, 10.0));
    /// ```
    pub fn absolute(self) -> Self
    where
        S: PartialOrd,
    {
        if self.start > self.end {
            self.invert()
        } else {
            self
        }
    }

    /// The Range that encompasses both self and the given Range.
    ///
    /// The returned Range's `start` will always be <= its `end`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(0.0, 3.0);
    /// let b = Range::new(7.0, 10.0);
    /// assert_eq!(a.max(b), Range::new(0.0, 10.0));
    ///
    /// let c = Range::new(-20.0, -30.0);
    /// let d = Range::new(5.0, -7.5);
    /// assert_eq!(c.max(d), Range::new(-30.0, 5.0));
    /// ```
    pub fn max(self, other: Self) -> Self
    where
        S: Float,
    {
        let start = self.start.min(self.end).min(other.start).min(other.end);
        let end = self.start.max(self.end).max(other.start).max(other.end);
        Range::new(start, end)
    }

    /// The Range that represents the range of the overlap between two Ranges if there is some.
    ///
    /// Note that If one end of `self` aligns exactly with the opposite end of `other`, `Some`
    /// `Range` will be returned with a magnitude of `0.0`. This is useful for algorithms that
    /// involve calculating the visibility of widgets, as it allows for including widgets whose
    /// bounding box may be a one dimensional straight line.
    ///
    /// The returned `Range`'s `start` will always be <= its `end`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(0.0, 6.0);
    /// let b = Range::new(4.0, 10.0);
    /// assert_eq!(a.overlap(b), Some(Range::new(4.0, 6.0)));
    ///
    /// let c = Range::new(10.0, -30.0);
    /// let d = Range::new(-5.0, 20.0);
    /// assert_eq!(c.overlap(d), Some(Range::new(-5.0, 10.0)));
    ///
    /// let e = Range::new(0.0, 2.5);
    /// let f = Range::new(50.0, 100.0);
    /// assert_eq!(e.overlap(f), None);
    /// ```
    pub fn overlap(mut self, mut other: Self) -> Option<Self>
    where
        S: PartialOrd + Sub<S, Output = S> + Zero,
    {
        self = self.absolute();
        other = other.absolute();
        let start = math::partial_max(self.start, other.start);
        let end = math::partial_min(self.end, other.end);
        let magnitude = end - start;
        if magnitude >= S::zero() {
            Some(Range::new(start, end))
        } else {
            None
        }
    }

    /// The Range that encompasses both self and the given Range.
    ///
    /// The same as [**Range::max**](./struct.Range.html#method.max) but retains `self`'s original
    /// direction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(0.0, 3.0);
    /// let b = Range::new(7.0, 10.0);
    /// assert_eq!(a.max_directed(b), Range::new(0.0, 10.0));
    ///
    /// let c = Range::new(-20.0, -30.0);
    /// let d = Range::new(5.0, -7.5);
    /// assert_eq!(c.max_directed(d), Range::new(5.0, -30.0));
    /// ```
    pub fn max_directed(self, other: Self) -> Self
    where
        S: Float,
    {
        if self.start <= self.end {
            self.max(other)
        } else {
            self.max(other).invert()
        }
    }

    /// Is the given scalar within our range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let range = Range::new(0.0, 10.0);
    /// assert!(range.contains(5.0));
    /// assert!(!range.contains(12.0));
    /// assert!(!range.contains(-1.0));
    /// assert!(range.contains(0.0));
    /// assert!(range.contains(10.0));
    /// ```
    pub fn contains(&self, pos: S) -> bool
    where
        S: PartialOrd,
    {
        let Range { start, end } = self.absolute();
        start <= pos && pos <= end
    }

    /// Round the values at both ends of the Range and return the result.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.25, 9.5).round(), Range::new(0.0, 10.0));
    /// assert_eq!(Range::new(4.95, -5.3).round(), Range::new(5.0, -5.0));
    /// ```
    pub fn round(self) -> Self
    where
        S: Float,
    {
        Self::new(self.start.round(), self.end.round())
    }

    /// Floor the values at both ends of the Range and return the result.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.25, 9.5).floor(), Range::new(0.0, 9.0));
    /// assert_eq!(Range::new(4.95, -5.3).floor(), Range::new(4.0, -6.0));
    /// ```
    pub fn floor(self) -> Self
    where
        S: Float,
    {
        Self::new(self.start.floor(), self.end.floor())
    }

    /// The Range with some padding given to the `start` value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad_start(2.0), Range::new(2.0, 10.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad_start(2.0), Range::new(8.0, 0.0));
    /// ```
    pub fn pad_start(mut self, pad: S) -> Self
    where
        S: Add<Output = S> + Neg<Output = S> + PartialOrd,
    {
        let new_start = self.start + if self.start <= self.end { pad } else { -pad };
        self.start = new_start;
        self
    }

    /// The Range with some padding given to the `end` value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad_end(2.0), Range::new(0.0, 8.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad_end(2.0), Range::new(10.0, 2.0));
    /// ```
    pub fn pad_end(mut self, pad: S) -> Self
    where
        S: Add<Output = S> + Neg<Output = S> + PartialOrd,
    {
        let new_end = self.end + if self.start <= self.end { -pad } else { pad };
        self.end = new_end;
        self
    }

    /// The Range with some given padding to be applied to each end.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad(2.0), Range::new(2.0, 8.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad(2.0), Range::new(8.0, 2.0));
    /// ```
    pub fn pad(self, pad: S) -> Self
    where
        S: Add<Output = S> + Neg<Output = S> + PartialOrd,
    {
        self.pad_start(pad).pad_end(pad)
    }

    /// The Range with some padding given for each end.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad_ends(1.0, 2.0), Range::new(1.0, 8.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad_ends(4.0, 3.0), Range::new(6.0, 3.0));
    /// ```
    pub fn pad_ends(self, start: S, end: S) -> Self
    where
        S: Add<Output = S> + Neg<Output = S> + PartialOrd,
    {
        self.pad_start(start).pad_end(end)
    }

    /// Clamp the given value to the range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).clamp_value(7.0), 5.0);
    /// assert_eq!(Range::new(5.0, -2.5).clamp_value(-3.0), -2.5);
    /// assert_eq!(Range::new(5.0, 10.0).clamp_value(0.0), 5.0);
    /// ```
    pub fn clamp_value(&self, value: S) -> S
    where
        S: PartialOrd,
    {
        math::clamp(value, self.start, self.end)
    }

    /// Stretch the end that is closest to the given value only if it lies outside the Range.
    ///
    /// The resulting Range will retain the direction of the original range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(2.5, 5.0);
    /// assert_eq!(a.stretch_to_value(10.0), Range::new(2.5, 10.0));
    /// assert_eq!(a.stretch_to_value(0.0), Range::new(0.0, 5.0));
    ///
    /// let b = Range::new(0.0, -5.0);
    /// assert_eq!(b.stretch_to_value(10.0), Range::new(10.0, -5.0));
    /// assert_eq!(b.stretch_to_value(-10.0), Range::new(0.0, -10.0));
    /// ```
    pub fn stretch_to_value(self, value: S) -> Self
    where
        S: PartialOrd,
    {
        let Range { start, end } = self;
        if start <= end {
            if value < start {
                Range { start: value, end }
            } else if value > end {
                Range { start, end: value }
            } else {
                self
            }
        } else if value < end {
            Range { start, end: value }
        } else if value > start {
            Range { start: value, end }
        } else {
            self
        }
    }

    /// Does `self` have the same direction as `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// assert!(Range::new(0.0, 1.0).has_same_direction(Range::new(100.0, 200.0)));
    /// assert!(Range::new(0.0, -5.0).has_same_direction(Range::new(-2.5, -6.0)));
    /// assert!(!Range::new(0.0, 5.0).has_same_direction(Range::new(2.5, -2.5)));
    /// ```
    pub fn has_same_direction(self, other: Self) -> bool
    where
        S: PartialOrd,
    {
        let self_direction = self.start <= self.end;
        let other_direction = other.start <= other.end;
        self_direction == other_direction
    }

    /// Align the `start` of `self` to the `start` of the `other` **Range**.
    ///
    /// If the direction of `other` is different to `self`, `self`'s `end` will be aligned to the
    /// `start` of `other` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_start_of(b), Range::new(0.0, 5.0));
    /// assert_eq!(b.align_start_of(a), Range::new(2.5, 12.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_start_of(d), Range::new(0.0, -5.0));
    /// assert_eq!(d.align_start_of(c), Range::new(-7.5, 2.5));
    /// ```
    pub fn align_start_of(self, other: Self) -> Self
    where
        S: PartialOrd + Add<S, Output = S> + Sub<S, Output = S>,
    {
        let diff = if self.has_same_direction(other) {
            other.start - self.start
        } else {
            other.start - self.end
        };
        self.shift(diff)
    }

    /// Align the `end` of `self` to the `end` of the `other` **Range**.
    ///
    /// If the direction of `other` is different to `self`, `self`'s `start` will be aligned to the
    /// `end` of `other` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_end_of(b), Range::new(5.0, 10.0));
    /// assert_eq!(b.align_end_of(a), Range::new(-2.5, 7.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_end_of(d), Range::new(5.0, 0.0));
    /// assert_eq!(d.align_end_of(c), Range::new(-2.5, 7.5));
    /// ```
    pub fn align_end_of(self, other: Self) -> Self
    where
        S: PartialOrd + Add<S, Output = S> + Sub<S, Output = S>,
    {
        let diff = if self.has_same_direction(other) {
            other.end - self.end
        } else {
            other.end - self.start
        };
        self.shift(diff)
    }

    /// Align the middle of `self` to the middle of the `other` **Range**.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(0.0, 5.0);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_middle_of(b), Range::new(2.5, 7.5));
    /// assert_eq!(b.align_middle_of(a), Range::new(-2.5, 7.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-10.0, 0.0);
    /// assert_eq!(c.align_middle_of(d), Range::new(-2.5, -7.5));
    /// assert_eq!(d.align_middle_of(c), Range::new(-5.0, 5.0));
    /// ```
    pub fn align_middle_of(self, other: Self) -> Self
    where
        S: Float,
    {
        let diff = other.middle() - self.middle();
        self.shift(diff)
    }

    /// Aligns the `start` of `self` with the `end` of `other`.
    ///
    /// If the directions are opposite, aligns the `end` of self with the `end` of `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_after(b), Range::new(10.0, 15.0));
    /// assert_eq!(b.align_after(a), Range::new(7.5, 17.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_after(d), Range::new(10.0, 5.0));
    /// assert_eq!(d.align_after(c), Range::new(-12.5, -2.5));
    /// ```
    pub fn align_after(self, other: Self) -> Self
    where
        S: PartialOrd + Add<S, Output = S> + Sub<S, Output = S>,
    {
        let diff = if self.has_same_direction(other) {
            other.end - self.start
        } else {
            other.end - self.end
        };
        self.shift(diff)
    }

    /// Aligns the `end` of `self` with the `start` of `other`.
    ///
    /// If the directions are opposite, aligns the `start` of self with the `start` of `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_before(b), Range::new(-5.0, 0.0));
    /// assert_eq!(b.align_before(a), Range::new(-7.5, 2.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_before(d), Range::new(-5.0, -10.0));
    /// assert_eq!(d.align_before(c), Range::new(2.5, 12.5));
    /// ```
    pub fn align_before(self, other: Self) -> Self
    where
        S: PartialOrd + Add<S, Output = S> + Sub<S, Output = S>,
    {
        let diff = if self.has_same_direction(other) {
            other.start - self.end
        } else {
            other.start - self.start
        };
        self.shift(diff)
    }

    /// Align `self` to `other` along the *x* axis in accordance with the given `Align` variant.
    pub fn align_to(self, align: Align, other: Self) -> Self
    where
        S: Float,
    {
        match align {
            Align::Start => self.align_start_of(other),
            Align::Middle => self.align_middle_of(other),
            Align::End => self.align_end_of(other),
        }
    }

    /// The closest **Edge** of `self` to the given `scalar`.
    ///
    /// Returns **Start** if the distance between both **Edge**s is equal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nannou_core as nannou;
    /// use nannou::geom::{Edge, Range};
    ///
    /// assert_eq!(Range::new(0.0, 10.0).closest_edge(4.0), Edge::Start);
    /// assert_eq!(Range::new(0.0, 10.0).closest_edge(7.0), Edge::End);
    /// assert_eq!(Range::new(0.0, 10.0).closest_edge(5.0), Edge::Start);
    /// ```
    pub fn closest_edge(&self, scalar: S) -> Edge
    where
        S: PartialOrd + Sub<S, Output = S>,
    {
        let Range { start, end } = *self;
        let start_diff = if scalar < start {
            start - scalar
        } else {
            scalar - start
        };
        let end_diff = if scalar < end {
            end - scalar
        } else {
            scalar - end
        };
        if start_diff <= end_diff {
            Edge::Start
        } else {
            Edge::End
        }
    }
}
