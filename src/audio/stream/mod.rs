use audio::sample::Sample;
use audio::{cpal, Device};
use std;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{self, AtomicBool};
use std::sync::{mpsc, Arc, Mutex};

/// Items related to output audio streams.
pub mod output;

/// Items related to input audio streams.
pub mod input;

/// Items related to duplex (synchronised input/output) audio streams.
///
/// *Progress is currently pending implementation of input audio streams in CPAL.*
pub mod duplex {}

/// The default sample rate used for output, input and duplex streams if possible.
pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

/// The type of function accepted for model updates.
pub type UpdateFn<M> = FnOnce(&mut M) + Send + 'static;

pub(crate) type ProcessFn = FnMut(cpal::StreamData) + 'static + Send;
pub(crate) type ProcessFnMsg = (cpal::StreamId, Box<ProcessFn>);

// State that is updated and run on the `cpal::EventLoop::run` thread.
pub(crate) struct LoopContext {
    // A channel for receiving callback functions for newly spawned streams.
    process_fn_rx: mpsc::Receiver<ProcessFnMsg>,
    // A map from StreamIds to their associated buffer processing functions.
    process_fns: HashMap<cpal::StreamId, Box<ProcessFn>>,
}

/// A clone-able handle around an audio stream.
pub struct Stream<M> {
    /// A channel for sending model updates to the audio thread.
    update_tx: mpsc::Sender<Box<FnMut(&mut M) + 'static + Send>>,
    /// A channel used for sending through a new buffer processing callback to the event loop.
    process_fn_tx: mpsc::Sender<ProcessFnMsg>,
    /// Data shared between each `Stream` handle to a single stream.
    shared: Arc<Shared<M>>,
    /// The format with which the stream was created.
    cpal_format: cpal::Format,
}

// Data shared between each `Stream` handle to a single stream.
struct Shared<M> {
    // The user's audio model
    model: Arc<Mutex<Option<M>>>,
    // A unique ID associated with this stream on the cpal EventLoop.
    stream_id: cpal::StreamId,
    // A handle to the CPAL audio event loop.
    event_loop: Arc<cpal::EventLoop>,
    // Whether or not the stream is currently paused.
    is_paused: AtomicBool,
}

/// Stream building parameters that are common between input and output streams.
pub struct Builder<M, S = f32> {
    pub(crate) event_loop: Arc<cpal::EventLoop>,
    pub(crate) process_fn_tx: mpsc::Sender<ProcessFnMsg>,
    pub model: M,
    pub sample_rate: Option<u32>,
    pub channels: Option<usize>,
    pub frames_per_buffer: Option<usize>,
    pub device: Option<Device>,
    pub(crate) sample_format: PhantomData<S>,
}

#[derive(Debug)]
pub enum BuildError {
    DefaultDevice,
    FormatEnumeration(cpal::FormatsEnumerationError),
    Creation(cpal::CreationError),
}

impl LoopContext {
    /// Create a new loop context.
    pub fn new(process_fn_rx: mpsc::Receiver<ProcessFnMsg>) -> Self {
        let process_fns = HashMap::new();
        LoopContext {
            process_fn_rx,
            process_fns,
        }
    }

    /// Process the given buffer with the stream at the given ID.
    pub fn process(&mut self, stream_id: cpal::StreamId, data: cpal::StreamData) {
        // Collect any pending stream process fns.
        for (stream_id, proc_fn) in self.process_fn_rx.try_iter() {
            self.process_fns.insert(stream_id, proc_fn);
        }

        // Process the data using the stream at the given ID.
        if let Some(proc_fn) = self.process_fns.get_mut(&stream_id) {
            proc_fn(data);
        // If there is not yet a buffer processing function, just silence the output buffer.
        } else {
            if let cpal::StreamData::Output { mut buffer } = data {
                fn silence<S: Sample>(slice: &mut [S]) {
                    for sample in slice {
                        *sample = S::equilibrium();
                    }
                }
                match buffer {
                    cpal::UnknownTypeOutputBuffer::U16(ref mut buffer) => silence(buffer),
                    cpal::UnknownTypeOutputBuffer::I16(ref mut buffer) => silence(buffer),
                    cpal::UnknownTypeOutputBuffer::F32(ref mut buffer) => silence(buffer),
                }
            }
        }
    }
}

impl<M> Stream<M> {
    /// Command the audio device to start processing this stream.
    ///
    /// Calling this will activate capturing/rendering, in turn calling the given audio
    /// capture/render function.
    ///
    /// Has no effect if the stream is already running.
    pub fn play(&self) {
        self.shared.play()
    }

    /// Command the audio device to stop processing this stream.
    ///
    /// Calling this will pause rendering/capturing.
    ///
    /// Has no effect is the stream was already paused.
    pub fn pause(&self) {
        self.shared.pause()
    }

    /// Whether or not the stream is currently playing.
    pub fn is_playing(&self) -> bool {
        self.shared.is_playing()
    }

    /// Whether or not the stream is currently paused.
    pub fn is_paused(&self) -> bool {
        self.shared.is_paused()
    }

    /// Send the given model update to the audio thread to be applied ASAP.
    ///
    /// If the audio is currently rendering, the update will be applied immediately after the
    /// function call completes.
    ///
    /// If the stream is currently paused, the update will be applied immediately.
    ///
    /// **Note:** This function will be applied on the real-time audio thread so users should
    /// avoid performing any kind of I/O, locking, blocking, (de)allocations or anything that
    /// may run for an indeterminate amount of time.
    pub fn send<F>(
        &self,
        update: F,
    ) -> Result<(), mpsc::SendError<Box<FnMut(&mut M) + Send + 'static>>>
    where
        F: FnOnce(&mut M) + Send + 'static,
    {
        // NOTE: The following code may mean that on extremely rare occasions an update does
        // not get applied for an indeterminate amount of time. This might be the case if a
        // stream is unpaused but becomes paused *immediately* after the `is_paused` atomic
        // condition is read as `false` - the update would be sent but the stream would be
        // paused and in turn the update will not get processed until the stream is unpaused
        // again. It would be nice to work out a solution to this that does not require
        // spawning another thread for each stream.

        // If the thread is currently paused, take the lock and immediately apply it as we know
        // there will be no contention with the audio thread.
        if self.shared.is_paused.load(atomic::Ordering::Relaxed) {
            if let Ok(mut guard) = self.shared.model.lock() {
                let mut model = guard.take().unwrap();
                update(&mut model);
                *guard = Some(model);
            }
        // Otherwise send the update to the audio thread.
        } else {
            // Move the `FnOnce` into a `FnMut` closure so that it can be called when it gets to
            // the audio thread. We do this as it's currently not possible to call a `Box<FnOnce>`,
            // as `FnOnce`'s `call` method takes `self` by value and thus is technically not object
            // safe.
            let mut update_opt = Some(update);
            let update_fn = move |audio: &mut M| {
                if let Some(update) = update_opt.take() {
                    update(audio);
                }
            };
            self.update_tx.send(Box::new(update_fn))?;
        }

        Ok(())
    }

    /// The format with which the inner CPAL stream was created.
    ///
    /// This **should** match the actual stream format that is running. If not, there may be a bug
    /// in CPAL. However, note that if the `sample_format` does not match, this just means that
    /// `nannou` is doing a conversion behind the scenes as the hardware itself does not support
    /// the target format.
    pub fn cpal_format(&self) -> &cpal::Format {
        &self.cpal_format
    }

    /// A reference to the unique ID associated with this stream.
    pub fn id(&self) -> &cpal::StreamId {
        &self.shared.stream_id
    }
}

impl<M> Shared<M> {
    fn play(&self) {
        self.event_loop.play_stream(self.stream_id.clone());
        self.is_paused.store(false, atomic::Ordering::Relaxed);
    }

    fn pause(&self) {
        self.is_paused.store(true, atomic::Ordering::Relaxed);
        self.event_loop.pause_stream(self.stream_id.clone());
    }

    fn is_playing(&self) -> bool {
        !self.is_paused.load(atomic::Ordering::Relaxed)
    }

    fn is_paused(&self) -> bool {
        self.is_paused.load(atomic::Ordering::Relaxed)
    }
}

impl<M> Clone for Stream<M> {
    fn clone(&self) -> Self {
        let update_tx = self.update_tx.clone();
        let process_fn_tx = self.process_fn_tx.clone();
        let shared = self.shared.clone();
        let cpal_format = self.cpal_format.clone();
        Stream {
            update_tx,
            process_fn_tx,
            shared,
            cpal_format,
        }
    }
}

impl<M> Drop for Shared<M> {
    fn drop(&mut self) {
        if self.is_playing() {
            self.pause();
        }
        self.event_loop.destroy_stream(self.stream_id.clone());
    }
}

impl From<cpal::CreationError> for BuildError {
    fn from(e: cpal::CreationError) -> Self {
        BuildError::Creation(e)
    }
}

impl From<cpal::FormatsEnumerationError> for BuildError {
    fn from(e: cpal::FormatsEnumerationError) -> Self {
        BuildError::FormatEnumeration(e)
    }
}

impl std::error::Error for BuildError {
    fn description(&self) -> &str {
        match *self {
            BuildError::DefaultDevice => "Failed to get default device",
            BuildError::FormatEnumeration(ref err) => err.description(),
            BuildError::Creation(ref err) => err.description(),
        }
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}

// Checks if the target format has a CPAL sample format equivalent.
//
// If so, we can use the sample format to target a stream format that already has a
// matchingn sample format.
//
// Otherwise we'll just fall back to the default sample format and do a conversionn.
fn cpal_sample_format<S: Any>() -> Option<cpal::SampleFormat> {
    let type_id = TypeId::of::<S>();
    if type_id == TypeId::of::<f32>() {
        Some(cpal::SampleFormat::F32)
    } else if type_id == TypeId::of::<i16>() {
        Some(cpal::SampleFormat::I16)
    } else if type_id == TypeId::of::<u16>() {
        Some(cpal::SampleFormat::U16)
    } else {
        None
    }
}

// Nannou allows the user to optionally specify each part of the stream format.
//
// This function is used to determine whether or not a supported format yielded by CPAL matches
// the requested format parameters specified by the user. If the supported format matches, a
// compatible `Format` is returned.
fn matching_supported_formats(
    mut supported_format: cpal::SupportedFormat,
    sample_format: Option<cpal::SampleFormat>,
    channels: Option<usize>,
    sample_rate: Option<cpal::SampleRate>,
) -> Option<cpal::Format> {
    // Check for a matching sample format.
    if let Some(sample_format) = sample_format {
        if supported_format.data_type != sample_format {
            return None;
        }
    }
    // Check for a matching number of channels.
    //
    // If there are more than enough channels, truncate the `SupportedFormat` to match.
    if let Some(channels) = channels {
        let supported_channels = supported_format.channels as usize;
        if supported_channels < channels {
            return None;
        } else if supported_channels > channels {
            supported_format.channels = channels as u16;
        }
    }
    // Check the sample rate.
    if let Some(sample_rate) = sample_rate {
        if supported_format.min_sample_rate > sample_rate
            || supported_format.max_sample_rate < sample_rate
        {
            return None;
        }
        let mut format = supported_format.with_max_sample_rate();
        format.sample_rate = sample_rate;
        return Some(format);
    }
    Some(supported_format.with_max_sample_rate())
}

// Given some audio device find the supported stream format that best matches the given optional
// format parameters (specified by the user).
fn find_best_matching_format<F>(
    device: &cpal::Device,
    mut sample_format: Option<cpal::SampleFormat>,
    channels: Option<usize>,
    mut sample_rate: Option<cpal::SampleRate>,
    default_format: Option<cpal::Format>,
    supported_formats: F,
) -> Result<Option<cpal::Format>, cpal::FormatsEnumerationError>
where
    F: Fn(&cpal::Device) -> Result<Vec<cpal::SupportedFormat>, cpal::FormatsEnumerationError>,
{
    loop {
        {
            // First, see if the default format satisfies the request.
            if let Some(ref format) = default_format {
                if sample_format == Some(format.data_type)
                    && channels
                        .map(|ch| ch <= format.channels as usize)
                        .unwrap_or(true)
                    && sample_rate == Some(format.sample_rate)
                {
                    return Ok(Some(format.clone()));
                }
            }

            // Otherwise search through all supported formats for compatible formats.
            let stream_formats = supported_formats(device)?.into_iter().filter_map(|fmt| {
                matching_supported_formats(fmt, sample_format, channels, sample_rate)
            });

            // Find the supported format with the most channels (this will always be the target
            // number of channels if some specific target number was specified as all other numbers
            // will have been filtered out already).
            if let Some(format) = stream_formats.max_by_key(|fmt| fmt.channels) {
                return Ok(Some(format));
            }
        }

        // If there are no matching formats with the target sample_format, drop the requirement
        // and we'll do a conversion from the default format instead.
        let default_sample_format = default_format.as_ref().map(|f| f.data_type);
        if sample_format.is_some() && sample_format != default_sample_format {
            sample_format = default_sample_format;
        // Otherwise if nannou's default target sample rate is set because the user didn't
        // specify a sample rate, try and fall back to a supported sample rate in case this was
        // the reason we could not find a supported format.
        } else if sample_rate == Some(cpal::SampleRate(DEFAULT_SAMPLE_RATE)) {
            let cpal_default_sample_rate = default_format.as_ref().map(|fmt| fmt.sample_rate);
            let nannou_default_sample_rate = Some(cpal::SampleRate(DEFAULT_SAMPLE_RATE));
            sample_rate = if cpal_default_sample_rate != nannou_default_sample_rate {
                cpal_default_sample_rate
            } else {
                None
            };
        } else {
            return Ok(None);
        }
    }
}
