//! Helper items related to working with sequences of musical bars.

use period::Period;
use time_calc as time;

/// An iterator that converts some iterator of ticks to their simplest possible musical divisions.
///
/// NOTE: This is ported from an old version of jen/core.
#[derive(Clone)]
pub struct SimplestDivisions<I> {
    ppqn: time::Ppqn,
    duration_ticks: time::Ticks,
    ticks_iter: I,
}

/// An iterator yielding each bar's time signature along with its starting position in ticks.
#[derive(Clone)]
pub struct WithStarts<I> {
    bars: I,
    next_start: time::Ticks,
    ppqn: time::Ppqn,
}

/// Convert an iterator yielding bar time signatures into their consecutive periods over time.
#[derive(Clone)]
pub struct Periods<I> {
    bars_with_starts: I,
    ppqn: time::Ppqn,
}

impl<I> SimplestDivisions<I> {
    /// Produce an iterator that converts some iterator of ticks to their simplest possible musical
    /// divisions.
    pub fn new<It>(ticks: It, ppqn: time::Ppqn, duration_ticks: time::Ticks) -> Self
    where
        It: IntoIterator<Item = time::Ticks, IntoIter = I>,
        It::IntoIter: Iterator<Item = time::Ticks>,
    {
        let ticks_iter = ticks.into_iter();
        SimplestDivisions {
            ticks_iter,
            ppqn,
            duration_ticks,
        }
    }
}

impl<I> WithStarts<I> {
    /// Convert an iterator yielding a time signature each bar to also yield the ticks at which
    /// that bar would begin.
    ///
    /// Assumes the first bar starts at `Ticks(0)`.
    pub fn new<It>(bars: It, ppqn: time::Ppqn) -> Self
    where
        It: IntoIterator<Item = time::TimeSig, IntoIter = I>,
        It::IntoIter: Iterator<Item = time::TimeSig>,
    {
        let bars = bars.into_iter();
        WithStarts {
            bars,
            ppqn,
            next_start: time::Ticks(0),
        }
    }
}

impl<I> Periods<WithStarts<I>> {
    pub fn new<It>(bars: It, ppqn: time::Ppqn) -> Self
    where
        It: IntoIterator<Item = time::TimeSig, IntoIter = I>,
        It::IntoIter: Iterator<Item = time::TimeSig>,
    {
        Periods {
            bars_with_starts: WithStarts::new(bars, ppqn),
            ppqn,
        }
    }
}

impl<I> Iterator for SimplestDivisions<I>
where
    I: Iterator<Item = time::Ticks>,
{
    type Item = Option<time::Division>;
    fn next(&mut self) -> Option<Option<time::Division>> {
        match self.ticks_iter.next() {
            None => None,
            Some(ticks) => {
                // If the ticks exceeds the duration of our bar, we'll stop iteration.
                if ticks > self.duration_ticks {
                    return None;
                }

                // If the ticks is 0, we can assume we're on the start of the Bar.
                if ticks == time::Ticks(0) {
                    return Some(Some(time::Division::Bar));
                }

                // We'll start at a `Minim` division and zoom in until we find a division that
                // would divide our ticks and return a whole number.
                let mut div = time::Division::Minim;
                while div.to_u8() <= time::Division::OneThousandTwentyFourth.to_u8() {
                    let div_in_beats = 2.0f64.powi(time::Division::Beat as i32 - div as i32);
                    let div_in_ticks = time::Ticks((div_in_beats * self.ppqn as f64).floor() as _);
                    if ticks % div_in_ticks == time::Ticks(0) {
                        return Some(Some(div));
                    }
                    div = div
                        .zoom_in(1)
                        .expect("Zoomed in too far when finding the simplest div");
                }

                // If we didn't find any matching divisions, we'll indicate this by returning None.
                Some(None)
            }
        }
    }
}

impl<I> Iterator for WithStarts<I>
where
    I: Iterator<Item = time::TimeSig>,
{
    type Item = (time::TimeSig, time::Ticks);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ts) = self.bars.next() {
            let ticks = ts.ticks_per_bar(self.ppqn);
            let start = self.next_start;
            self.next_start += ticks;
            return Some((ts, start));
        }
        None
    }
}

impl<I> Iterator for Periods<I>
where
    I: Iterator<Item = (time::TimeSig, time::Ticks)>,
{
    type Item = Period;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((ts, start)) = self.bars_with_starts.next() {
            let duration = ts.ticks_per_bar(self.ppqn);
            let end = start + duration;
            return Some(Period { start, end });
        }
        None
    }
}
