use crate::Device;
use cpal::traits::StreamTrait;
use std;
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::atomic::{self, AtomicBool};
use std::sync::{mpsc, Arc, Mutex};
use thiserror::Error;

/// Items related to input audio streams.
pub mod input;
/// Items related to output audio streams.
pub mod output;
/// Items related to duplex (synchronised input/output) audio streams.
///
/// *Progress is currently pending implementation of duplex audio streams in CPAL.*
pub mod duplex {}

/// Called by the audio host in the case that an error occurs on an audio stream thread.
pub trait ErrorFn<M>: Fn(&mut M, cpal::StreamError) {}

/// The type of function accepted for model updates.
pub type UpdateFn<M> = dyn FnOnce(&mut M) + Send + 'static;
/// The default stream error function type used when unspecified.
pub type DefaultErrorFn<M> = fn(&mut M, err: cpal::StreamError);

/// A clone-able handle around an audio stream.
pub struct Stream<M> {
    /// A channel for sending model updates to the audio thread.
    update_tx: mpsc::Sender<Box<dyn FnMut(&mut M) + 'static + Send>>,
    /// Data shared between each `Stream` handle to a single stream.
    shared: Arc<Shared<M>>,
    /// The stream config with which the stream was created.
    cpal_config: cpal::StreamConfig,
}

// Data shared between each `Stream` handle to a single stream.
struct Shared<M> {
    // The CPAL stream handle.
    stream: cpal::Stream,
    // The user's audio model
    model: Arc<Mutex<Option<M>>>,
    // Whether or not the stream is currently paused.
    is_paused: AtomicBool,
}

/// Stream building parameters that are common between input and output streams.
pub struct Builder<M, S = f32> {
    pub(crate) host: Arc<cpal::Host>,
    pub model: M,
    pub sample_rate: Option<u32>,
    pub channels: Option<usize>,
    pub frames_per_buffer: Option<usize>,
    pub device_buffer_size: Option<cpal::BufferSize>,
    pub device: Option<Device>,
    pub(crate) sample_format: PhantomData<S>,
}

/// Errors that might occur when attempting to build a stream.
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("failed to get default device")]
    DefaultDevice,
    #[error("failed to enumerate available configs: {err}")]
    SupportedStreamConfigs {
        err: cpal::SupportedStreamConfigsError,
    },
    #[error("failed to build stream: {err}")]
    BuildStream { err: cpal::BuildStreamError },
}

struct DesiredStreamConfig {
    /// Sample format specified by the user via the `S` sample type.
    sample_format: Option<cpal::SampleFormat>,
    /// Channel count specified by the user.
    channels: Option<usize>,
    /// The user's if specified, otherwise is `DEFAULT_SAMPLE_RATE`.
    sample_rate: Option<cpal::SampleRate>,
    /// Desired device buffer size specified by the user.
    device_buffer_size: Option<cpal::BufferSize>,
}

/// The default sample rate used for output, input and duplex streams if possible.
pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

impl<M> Stream<M> {
    /// Command the audio device to start processing this stream.
    ///
    /// Calling this will activate capturing/rendering, in turn calling the given audio
    /// capture/render function.
    ///
    /// Has no effect if the stream is already running.
    pub fn play(&self) -> Result<(), cpal::PlayStreamError> {
        self.shared.play()
    }

    /// Command the audio device to stop processing this stream.
    ///
    /// Calling this will pause rendering/capturing.
    ///
    /// Has no effect is the stream was already paused.
    pub fn pause(&self) -> Result<(), cpal::PauseStreamError> {
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
    ) -> Result<(), mpsc::SendError<Box<dyn FnMut(&mut M) + Send + 'static>>>
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

    /// The config with which the inner CPAL stream was created.
    ///
    /// This **should** match the actual stream config that is running. If not, there may be a bug
    /// in CPAL. However, note that if the `sample_format` does not match, this just means that
    /// `nannou` is doing a conversion behind the scenes as the hardware itself does not support
    /// the target format.
    pub fn cpal_config(&self) -> &cpal::StreamConfig {
        &self.cpal_config
    }
}

impl<M> Shared<M> {
    fn play(&self) -> Result<(), cpal::PlayStreamError> {
        self.stream.play()?;
        self.is_paused.store(false, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn pause(&self) -> Result<(), cpal::PauseStreamError> {
        self.stream.pause()?;
        self.is_paused.store(true, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn is_playing(&self) -> bool {
        !self.is_paused.load(atomic::Ordering::Relaxed)
    }

    fn is_paused(&self) -> bool {
        self.is_paused.load(atomic::Ordering::Relaxed)
    }
}

impl<M, F> ErrorFn<M> for F where F: Fn(&mut M, cpal::StreamError) {}

impl<M> Clone for Stream<M> {
    fn clone(&self) -> Self {
        let update_tx = self.update_tx.clone();
        let shared = self.shared.clone();
        let cpal_config = self.cpal_config.clone();
        Stream {
            update_tx,
            shared,
            cpal_config,
        }
    }
}

impl<M> Drop for Shared<M> {
    fn drop(&mut self) {
        if self.is_playing() {
            self.pause().ok();
        }
    }
}

impl From<cpal::BuildStreamError> for BuildError {
    fn from(err: cpal::BuildStreamError) -> Self {
        BuildError::BuildStream { err }
    }
}

impl From<cpal::SupportedStreamConfigsError> for BuildError {
    fn from(err: cpal::SupportedStreamConfigsError) -> Self {
        BuildError::SupportedStreamConfigs { err }
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

// Nannou allows the user to optionally specify each part of the stream config.
//
// This function is used to determine whether or not a supported config yielded by CPAL matches
// the requested config parameters specified by the user. If the supported config matches, a
// compatible `SupportedStreamConfig` is returned.
fn matching_supported_config(
    desired: &DesiredStreamConfig,
    supported_stream_config_range: &cpal::SupportedStreamConfigRange,
) -> Option<MatchingConfig> {
    // Check for a matching buffer size.
    if let Some(cpal::BufferSize::Fixed(size)) = desired.device_buffer_size {
        let supported_size = supported_stream_config_range.buffer_size();
        if let cpal::SupportedBufferSize::Range { min, max } = *supported_size {
            if size > max || size < min {
                return None;
            }
        }
    }

    // Check for a matching sample format.
    let supported_sample_format = supported_stream_config_range.sample_format();
    if let Some(sample_format) = desired.sample_format {
        if supported_sample_format != sample_format {
            return None;
        }
    }
    let sample_format = desired.sample_format.unwrap_or(supported_sample_format);

    // Check for a matching number of channels.
    if let Some(channels) = desired.channels {
        let supported_channels = supported_stream_config_range.channels() as usize;
        if supported_channels < channels {
            return None;
        }
    }

    // Check the sample rate.
    if let Some(sample_rate) = desired.sample_rate {
        if supported_stream_config_range.min_sample_rate() > sample_rate
            || supported_stream_config_range.max_sample_rate() < sample_rate
        {
            return None;
        }
        let mut config = supported_stream_config_range
            .clone()
            .with_sample_rate(sample_rate)
            .config();
        config.buffer_size = desired
            .device_buffer_size
            .clone()
            .unwrap_or(cpal::BufferSize::Default);
        let matching = MatchingConfig {
            config,
            sample_format,
        };
        return Some(matching);
    }

    // If we got this far, the user didn't specify a sample rate.
    let mut config = supported_stream_config_range
        .clone()
        .with_max_sample_rate()
        .config();
    config.buffer_size = desired
        .device_buffer_size
        .clone()
        .unwrap_or(cpal::BufferSize::Default);
    let matching = MatchingConfig {
        config,
        sample_format,
    };
    Some(matching)
}

fn desired_device_buffer_size_matches_default(
    desired: Option<&cpal::BufferSize>,
    supported: &cpal::SupportedBufferSize,
) -> bool {
    match desired {
        None | Some(&cpal::BufferSize::Default) => true,
        Some(&cpal::BufferSize::Fixed(size)) => match *supported {
            cpal::SupportedBufferSize::Range { min, max } if size >= min && size <= max => true,
            _ => false,
        },
    }
}

struct MatchingConfig {
    sample_format: cpal::SampleFormat,
    config: cpal::StreamConfig,
}

// Whether or not the desired stream config matches the default one.
fn desired_config_matches_default(
    desired: &DesiredStreamConfig,
    default: &cpal::SupportedStreamConfig,
) -> Option<MatchingConfig> {
    if desired.sample_format == Some(default.sample_format())
        && desired
            .channels
            .map(|ch| ch <= default.channels() as usize)
            .unwrap_or(true)
        && desired.sample_rate == Some(default.sample_rate())
        && desired_device_buffer_size_matches_default(
            desired.device_buffer_size.as_ref(),
            default.buffer_size(),
        )
    {
        let mut config = default.config();
        config.buffer_size = desired
            .device_buffer_size
            .clone()
            .unwrap_or(cpal::BufferSize::Default);
        let sample_format = desired.sample_format.unwrap_or(default.sample_format());
        let matching = MatchingConfig {
            config,
            sample_format,
        };
        Some(matching)
    } else {
        None
    }
}

// Given some audio device find the supported stream config that best matches the given optional
// config parameters (specified by the user).
fn find_best_matching_config<F>(
    device: &cpal::Device,
    mut desired: DesiredStreamConfig,
    default: Option<cpal::SupportedStreamConfig>,
    supported_configs: F,
) -> Result<Option<MatchingConfig>, cpal::SupportedStreamConfigsError>
where
    F: Fn(
        &cpal::Device,
    ) -> Result<Vec<cpal::SupportedStreamConfigRange>, cpal::SupportedStreamConfigsError>,
{
    // In the case that the user has not specified a sample rate, we want to try specifying a
    // reasonable default ourselves, otherwise CPAL can give back some extremely high frequency
    // ones by default that are generally less practical.
    let mut trying_default_sample_rate = false;
    if desired.sample_rate.is_none() {
        desired.sample_rate = Some(cpal::SampleRate(DEFAULT_SAMPLE_RATE));
        trying_default_sample_rate = true;
    }

    loop {
        {
            // First, see if the default config satisfies the request.
            if let Some(ref default) = default {
                if let Some(conf) = desired_config_matches_default(&desired, default) {
                    return Ok(Some(conf));
                }
            }

            // Otherwise search through all supported configs for compatible configs.
            let stream_configs = supported_configs(device)?
                .into_iter()
                .filter_map(|config| matching_supported_config(&desired, &config));

            // Find the supported config with the most channels (this will always be the target
            // number of channels if some specific target number was specified as all other numbers
            // will have been filtered out already).
            if let Some(matching) = stream_configs.max_by_key(|matching| matching.config.channels) {
                return Ok(Some(matching));
            }
        }

        // If there are no matching configs with the target sample_format, drop the requirement
        // and we'll do a conversion to the desired sample rate from the default config in the
        // stream callback ourselves.
        let default_sample_format = default.as_ref().map(|f| f.sample_format());
        if desired.sample_format.is_some() && desired.sample_format != default_sample_format {
            desired.sample_format = default_sample_format;
            continue;
        }

        // If we tried specifying our own default sample rate in the case that the user didn't
        // specify one, remove it and try again.
        if trying_default_sample_rate {
            trying_default_sample_rate = false;
            desired.sample_rate = None;
            continue;
        }

        // Otherwise, there are no matches for the request.
        return Ok(None);
    }
}

// The default error function used when unspecified.
pub(crate) fn default_error_fn<M>(_: &mut M, err: cpal::StreamError) {
    eprintln!("A `StreamError` occurred: {}", err);
}
