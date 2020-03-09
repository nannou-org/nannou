use std::ops::Sub;
use time_calc::Ticks;

/// A period of time in ticks.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Period<T = Ticks> {
    pub start: T,
    pub end: T,
}

impl<T> Period<T> {
    /// The duration of the period.
    pub fn duration(&self) -> T
    where
        T: Clone + Sub<T, Output = T>,
    {
        self.end.clone() - self.start.clone()
    }

    /// Does the given ticks fall within the period.
    #[inline]
    pub fn contains(&self, t: T) -> bool
    where
        T: PartialOrd + PartialEq,
    {
        t >= self.start && t < self.end
    }

    /// Whether or not self intersects with the other period.
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool
    where
        T: PartialOrd,
    {
        !(other.start > self.end || self.start > other.end)
    }
}
