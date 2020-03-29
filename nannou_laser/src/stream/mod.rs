pub mod frame;
pub mod raw;

/// The default rate at which the DAC should request points per second.
pub const DEFAULT_POINT_HZ: u32 = 10_000;

/// The default rate at which the DAC will yield frames of points.
pub const DEFAULT_FRAME_HZ: u32 = 60;

/// Builder parameters shared between the `raw` and `frame` signals.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    /// The DAC with which the stream should be established.
    pub dac: Option<crate::DetectedDac>,
    /// The initial rate at which the DAC should process points per second.
    ///
    /// By default this value is `stream::DEFAULT_POINT_HZ`.
    pub point_hz: Option<u32>,
    /// The maximum latency specified as a number of points.
    ///
    /// Each time the laser indicates its "fullness", the raw stream will request enough points
    /// from the render function to fill the DAC buffer up to `latency_points`.
    pub latency_points: Option<u32>,
}

/// Given a DAC point rate and a desired frame rate, determine how many points to generate per
/// frame.
pub fn points_per_frame(point_hz: u32, frame_hz: u32) -> u32 {
    point_hz / frame_hz
}
