pub use self::output::Output;

/// Items related to output audio streams.
pub mod output;

/// Items related to input audio streams.
///
/// *Progress is currently pending implementation of input audio streams in CPAL.*
pub mod input {}

/// Items related to duplex (synchronised input/output) audio streams.
///
/// *Progress is currently pending implementation of input audio streams in CPAL.*
pub mod duplex {}


/// The default sample rate used for output, input and duplex streams if possible.
pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

/// The type of function accepted for model updates.
pub type UpdateFn<M> = FnMut(&mut M) + Send + 'static;
