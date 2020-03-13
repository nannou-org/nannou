//! A multi-media timeline library.
//!
//! The primary type is **Timeline** - a conrod **Widget** implementation that can take an
//! arbitrary number of **Track**s in any order.

#[macro_use]
extern crate conrod_core;
#[macro_use]
extern crate conrod_derive;
extern crate envelope;
extern crate itertools;
extern crate num;
extern crate pitch_calc;
extern crate time_calc;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

pub use period::Period;
pub use playhead::Playhead;
pub use ruler::Ruler;
pub use timeline::{Context, Final, PinnedTracks, Timeline, Track, TrackStyle, Tracks};

pub mod bars;
mod diff; // temporary until diff.rs lands in iter-tools.
pub(crate) mod env;
pub mod period;
pub mod playhead;
mod ruler;
mod timeline;
pub mod track;

use time_calc as time;

/// The duration of a sequence of bars in ticks.
pub fn bars_duration_ticks<I>(bars: I, ppqn: time::Ppqn) -> time::Ticks
where
    I: IntoIterator<Item = time::TimeSig>,
{
    bars.into_iter()
        .fold(time::Ticks(0), |acc, ts| acc + ts.ticks_per_bar(ppqn))
}
