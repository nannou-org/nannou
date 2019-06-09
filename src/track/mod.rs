use conrod;
use core::time;
pub use self::piano_roll::PianoRoll;
pub use self::ruler::Ruler;

pub mod automation;
pub mod piano_roll;
pub mod ruler;

/// The default height used for tracks if none was specified by the user.
pub const DEFAULT_HEIGHT: conrod::Scalar = 70.0;

/// Widgets that may be set as the `Timeline`'s `Track`s.
pub trait Widget: conrod::Widget {

    /// Build the widget with the given playhead position and delta in ticks.
    ///
    /// If this method is not overridden, the playhead will be ignored.
    fn playhead(self, (time::Ticks, time::Ticks)) -> Self {
        self
    }

}
