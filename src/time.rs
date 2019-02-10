//! Extensions and utilities for working with time.

/// An extension for the `std::time::Duration` type providing some simple methods for easy access
/// to an `f64` representation of seconds, ms, mins, hrs, and other units of time.
///
/// While these measurements make it easier to work with sketches and artworks, it's worth noting
/// that resolution may be lost, especially at high values.
pub trait DurationF64 {
    /// A simple way of retrieving the duration in seconds.
    fn secs(&self) -> f64;

    /// A simple way of retrieving the duration in milliseconds.
    ///
    /// By default, this is implemented as `self.secs() * 1_000.0`.
    fn ms(&self) -> f64 {
        self.secs() * 1_000.0
    }

    /// A simple way of retrieving the duration as minutes.
    fn mins(&self) -> f64 {
        self.secs() / 60.0
    }

    /// A simple way of retrieving the duration as hrs.
    fn hrs(&self) -> f64 {
        self.secs() / 3_600.0
    }

    /// A simple way of retrieving the duration as days.
    fn days(&self) -> f64 {
        self.secs() / 86_400.0
    }

    /// A simple way of retrieving the duration as weeks.
    fn weeks(&self) -> f64 {
        self.secs() / 604_800.0
    }
}

impl DurationF64 for std::time::Duration {
    fn secs(&self) -> f64 {
        self.as_secs() as f64 + self.subsec_nanos() as f64 * 1e-9
    }
}
