use bars;
use conrod_core::Scalar;
use std::iter::{Enumerate, Zip};
use time_calc as time;

/// For converting a duration in bars to a viewable grid.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ruler {
    pub width_per_beat: Scalar,
    marker_step: (time::NumDiv, time::Division),
}

/// The information required to construct a new `Ruler` instance.
#[derive(Clone, Debug)]
pub struct RangeDescription<T> {
    pub ppqn: time::Ppqn,
    pub duration_ticks: time::Ticks,
    pub duration_bars: time::Bars,
    pub time_sigs: T,
}

// Iterators

/// An iterator that yields an iterator of marker positions (in ticks) for each Bar.
#[derive(Clone)]
pub struct MarkersInTicks<I> {
    ppqn: time::Ppqn,
    marker_step: (time::NumDiv, time::Division),
    bars_with_starts_enumerated: Enumerate<I>,
}

/// An iterator that yields each marker position on a ruler in ticks.
#[derive(Clone)]
pub struct BarMarkersInTicks {
    maybe_next: Option<time::Ticks>,
    marker_step_ticks: time::Ticks,
    end_ticks: time::Ticks,
}

/// An alias for markers in ticks per bar along with the TimeSig for each bar.
type MarkersInTicksWithBarsAndStarts<I> = Zip<MarkersInTicks<I>, I>;

/// An iterator that yields an iterator producing the simplest Division for each marker in a Bar.
#[derive(Clone)]
pub struct MarkersInDivisions<I> {
    ppqn: time::Ppqn,
    markers_in_ticks_with_bars_and_starts: MarkersInTicksWithBarsAndStarts<I>,
    marker_div: time::Division,
}

/// An iterator that yields the simplest Division for each marker in a Bar suitable for the Ruler.
#[derive(Clone)]
pub struct BarMarkersInDivisions {
    time_sig_bottom: u16,
    simplest_divisions: bars::SimplestDivisions<BarMarkersInTicks>,
    marker_div: time::Division,
}

// Implementations

impl Ruler {
    /// Constructor for a Ruler.
    pub fn new<T>(total_width: f64, desc: RangeDescription<T>) -> Ruler
    where
        T: Iterator<Item = time::TimeSig> + Clone,
    {
        let RangeDescription {
            ppqn,
            duration_ticks,
            duration_bars,
            time_sigs,
        } = desc;

        let total_ticks = duration_ticks;
        let total_beats = total_ticks.ticks() as f64 / ppqn as f64;
        let width_per_beat = total_width / total_beats;
        let total_num_bars = duration_bars.0 as usize;

        // If we don't have a duration, our Ruler is meaningless.
        if total_num_bars == 0 {
            return Ruler {
                width_per_beat: 0.0,
                marker_step: (0, time::Division::Bar),
            };
        }

        // In order to determine the measure we need, we first need to define a
        // resolution limit for the steps in the ruler.
        const MIN_STEP_WIDTH: f64 = 30.0;

        // If we find a suitable step as a number of bars, we'll bind it to this.
        let mut maybe_bars_step: Option<time::NumDiv> = None;

        // We'll start by checking steps of large numbers of bars first.
        let mut pow_two: u32 = 8;
        loop {
            use itertools::Itertools;

            let num_bars = 2usize.pow(pow_two);

            // We should only bother testing num_bars that are less than or equal to
            // our total number of bars.
            if num_bars > total_num_bars {
                if pow_two > 0 {
                    pow_two -= 1;
                    continue;
                } else {
                    break;
                }
            }

            // Check for the smallest step in terms of width that would be produced by a ruler
            // with markers divided by the current `num_bars`.
            let step_in_width = time_sigs
                .clone()
                .chunks_lazy(num_bars)
                .into_iter()
                .fold(::std::f64::MAX, |smallest_step, time_sigs| {
                    let step = time_sigs.fold(0.0, |total, ts| {
                        total + (ts.beats_per_bar() * width_per_beat)
                    });
                    step.min(smallest_step)
                });

            // If our step is still greater than the MIN_STEP_WIDTH, we'll keep
            // searching smaller and smaller num_bars steps.
            if step_in_width >= MIN_STEP_WIDTH {
                if pow_two > 0 {
                    pow_two -= 1;
                    continue;
                } else {
                    break;
                }
            } else {
                // Otherwise, we've found our step as a number of bars.
                pow_two += 1;
                let num_bars = 2usize.pow(pow_two);
                maybe_bars_step = Some(num_bars as time::NumDiv);
                break;
            }
        }

        // If maybe_bars_step is some, then we've already found our smallest step.
        // Otherwise, we still need to check bar divisions for a suitable step.
        let mut use_quaver_step = false;
        let mut maybe_div_step = None;
        if maybe_bars_step.is_none() {
            // First we need to check if the time_sig denominator measure would create
            // a suitable step, before trying smaller measures.
            let quaver_step_in_pixels = width_per_beat / 2.0;
            if quaver_step_in_pixels < MIN_STEP_WIDTH {
                use_quaver_step = true;
            } else {
                // We want to loop into finer resolutions by having a divider that
                // doubles every iteration. To do this, we'll raise two to some power
                // starting at 2.0 (4th of a beat aka semi_quaver) going smaller until
                // we reach the most suitable step.
                let mut pow_two = 2.0;
                loop {
                    let divider = 2.0f64.powf(pow_two);
                    let measure_step_in_pixels = width_per_beat / divider;
                    // If this measure step would be smaller, the previous measure step
                    // is the one that we're after.
                    if measure_step_in_pixels < MIN_STEP_WIDTH {
                        pow_two -= 1.0;
                        maybe_div_step = time::Division::Beat.zoom_in(pow_two as u8);
                        break;
                    }
                    pow_two += 1.0;
                }
            }
        }

        Ruler {
            width_per_beat: width_per_beat,
            marker_step: if let Some(div) = maybe_div_step {
                (1, div)
            } else if use_quaver_step {
                (1, time::Division::Quaver)
            } else if let Some(num) = maybe_bars_step {
                (num, time::Division::Bar)
            } else {
                unreachable!();
            },
        }
    }

    /// Produce an iterator that yields an iterator for each bar along with its start position in
    /// ticks.
    pub fn markers_in_ticks<I>(
        &self,
        bars: I,
        ppqn: time::Ppqn,
    ) -> MarkersInTicks<bars::WithStarts<I::IntoIter>>
    where
        I: IntoIterator<Item = time::TimeSig>,
        I::IntoIter: Clone,
    {
        let bars_with_starts = bars::WithStarts::new(bars, ppqn);
        MarkersInTicks {
            marker_step: self.marker_step,
            bars_with_starts_enumerated: bars_with_starts.into_iter().enumerate(),
            ppqn,
        }
    }

    /// Produce an iterator that yields an iterator for each bar that yields each marker's simplest
    /// division representation suitable for the Ruler.
    pub fn markers_in_divisions<I>(
        &self,
        bars: I,
        ppqn: time::Ppqn,
    ) -> MarkersInDivisions<bars::WithStarts<I::IntoIter>>
    where
        I: IntoIterator<Item = time::TimeSig>,
        I::IntoIter: Clone,
    {
        let bars = bars.into_iter();
        let bars_with_starts = bars::WithStarts::new(bars.clone(), ppqn);
        MarkersInDivisions {
            ppqn,
            markers_in_ticks_with_bars_and_starts: self
                .markers_in_ticks(bars, ppqn)
                .zip(bars_with_starts),
            marker_div: self.marker_step.1,
        }
    }

    /// Produces the number of visible markers on the `Ruler` for the given bars.
    pub fn marker_count<I>(&self, bars: I, ppqn: time::Ppqn) -> usize
    where
        I: IntoIterator<Item = time::TimeSig>,
        I::IntoIter: Clone,
    {
        // TODO: Could probably do this more efficiently, but this is easy for now.
        self.markers_in_ticks(bars, ppqn)
            .flat_map(|bar_markers| bar_markers)
            .count()
    }

    // /// The total spatial width representing the duration of a single tick.
    // pub fn width_per_tick(&self) -> Scalar {
    //     self.width_per_beat / core::PPQN as Scalar
    // }

    /// The fractional number of ticks that may fit within one unit of space.
    pub fn ticks_per_width(&self, ppqn: time::Ppqn) -> Scalar {
        (1.0 / self.width_per_beat) * ppqn as Scalar
    }
}

/// Maps the given `ticks` to some offset along a given `width`.
///
/// This is often used for translating `Ticks` values into useful `Scalar` coordinates.
pub fn x_offset_from_ticks(ticks: time::Ticks, total: time::Ticks, width: Scalar) -> Scalar {
    (ticks.ticks() as Scalar / total.ticks() as Scalar) * width - width / 2.0
}

// Iterator implementations.

impl<I> Iterator for MarkersInTicks<I>
where
    I: Iterator<Item = (time::TimeSig, time::Ticks)> + Clone,
{
    type Item = BarMarkersInTicks;
    fn next(&mut self) -> Option<BarMarkersInTicks> {
        self.bars_with_starts_enumerated
            .next()
            .map(|(i, (time_sig, start))| match self.marker_step {
                (n, time::Division::Bar) => {
                    let bar_ticks = time_sig.ticks_per_bar(self.ppqn);
                    BarMarkersInTicks {
                        maybe_next: if i % n as usize == 0 {
                            Some(start)
                        } else {
                            None
                        },
                        marker_step_ticks: bar_ticks,
                        end_ticks: start + bar_ticks,
                    }
                }
                (1, div) => BarMarkersInTicks {
                    maybe_next: Some(start),
                    marker_step_ticks: time::Measure(1, div, time::DivType::Whole)
                        .to_ticks(time_sig, self.ppqn),
                    end_ticks: start + time_sig.ticks_per_bar(self.ppqn),
                },
                _ => unreachable!(),
            })
    }
}

impl Iterator for BarMarkersInTicks {
    type Item = time::Ticks;
    fn next(&mut self) -> Option<time::Ticks> {
        match self.maybe_next {
            None => None,
            Some(this_marker_ticks) => {
                let next_marker_ticks = this_marker_ticks + self.marker_step_ticks;
                self.maybe_next = if next_marker_ticks < self.end_ticks {
                    Some(next_marker_ticks)
                } else {
                    None
                };
                Some(this_marker_ticks)
            }
        }
    }
}

impl<I> Iterator for MarkersInDivisions<I>
where
    I: Iterator<Item = (time::TimeSig, time::Ticks)> + Clone,
{
    type Item = BarMarkersInDivisions;
    fn next(&mut self) -> Option<BarMarkersInDivisions> {
        self.markers_in_ticks_with_bars_and_starts
            .next()
            .map(|(markers_in_ticks, (time_sig, start))| {
                let time_sig_bottom = time_sig.bottom;
                let markers_in_ticks = BarMarkersInTicks {
                    maybe_next: markers_in_ticks.maybe_next.map(|ticks| ticks - start),
                    end_ticks: markers_in_ticks.end_ticks - start,
                    ..markers_in_ticks
                };
                let ppqn = self.ppqn;
                let duration_ticks = time_sig.ticks_per_bar(ppqn);
                let ticks_iter = markers_in_ticks;
                let simplest_divisions = bars::SimplestDivisions::new(
                    ticks_iter,
                    ppqn,
                    duration_ticks,
                );
                BarMarkersInDivisions {
                    time_sig_bottom: time_sig_bottom,
                    simplest_divisions: simplest_divisions,
                    marker_div: self.marker_div,
                }
            })
    }
}

impl Iterator for BarMarkersInDivisions {
    type Item = time::Division;
    fn next(&mut self) -> Option<time::Division> {
        self.simplest_divisions.next().map(|maybe_div| {
            let div = maybe_div.expect("No simplest division found.");
            match self.time_sig_bottom {
                4 => match div {
                    time::Division::Bar | time::Division::Beat | time::Division::Quaver => div,
                    time::Division::Minim => time::Division::Beat,
                    _ => self.marker_div,
                },
                8 => match div {
                    time::Division::Bar | time::Division::Quaver => div,
                    time::Division::Minim | time::Division::Beat => time::Division::Quaver,
                    _ => self.marker_div,
                },
                _ => unreachable!(),
            }
        })
    }
}
