//! A multi-media timeline library.
//!
//! The primary type is **Timeline** - a conrod **Widget** implementation that can take an
//! arbitrary number of **Track**s in any order.

#[macro_use] extern crate conrod_core;
#[macro_use] extern crate conrod_derive;
extern crate itertools;
extern crate num;

pub use playhead::Playhead;
pub use ruler::Ruler;
pub use timeline::{Context, PinnedTracks, Tracks, Final, Timeline, Track, TrackStyle};

mod diff; // temporary until diff.rs lands in iter-tools.
pub mod playhead;
mod ruler;
mod timeline;
pub mod track;
