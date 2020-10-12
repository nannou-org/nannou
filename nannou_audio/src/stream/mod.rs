use crate::Device;
use cpal::traits::StreamTrait;
use dasp_sample::Sample;
use failure::Fail;
use std;
use std::any::{Any, TypeId};
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
pub type UpdateFn<M> = dyn FnOnce(&mut M) + Send + 'static;

pub(crate) type ProcessFn = dyn FnMut(cpal::StreamDataResult) + 'static + Send;
pub(crate) type ProcessFnMsg = Box<ProcessFn>;

/// A clone-able handle around an audio stream.
pub struct Stream<M> {
    /// A channel for sending model updates to the audio thread.
    update_tx: mpsc::Sender<Box<dyn FnMut(&mut M) + 'static + Send>>,
    /// A channel used for sending through a new buffer processing callback to the event loop.
    process_fn_tx: mpsc::Sender<ProcessFnMsg>,
    /// Data shared between each `Stream` handle to a single stream.
    shared: Arc<Shared<M>>,
    /// The stream config with which the stream was created.
    cpal_stream_config: cpal::SupportedStreamConfig,
}

// Data shared between each `Stream` handle to a single stream.
struct Shared<M> {
    // The user's audio model
    model: Arc<Mutex<Option<M>>>,
    // Whether or not the stream is currently paused.
    is_paused: AtomicBool,
}

/// Stream building parameters that are common between input and output streams.
pub struct Builder<M, S = f32> {
    pub(crate) host: Arc<cpal::Host>,
    pub(crate) process_fn_tx: mpsc::Sender<ProcessFnMsg>,
    pub model: M,
    pub sample_rate: Option<u32>,
    pub channels: Option<usize>,
    pub frames_per_buffer: Option<usize>,
    pub device: Option<Device>,
    pub(crate) sample_format: PhantomData<S>,
}

/// Errors that might occur when attempting to build a stream.
#[derive(Debug, Fail)]
pub enum BuildError {
    #[fail(display = "failed to get default device")]
    DefaultDevice,
    #[fail(display = "failed to enumerate available configs: {}", err)]
    SupportedStreamConfigs {
        err: cpal::SupportedStreamConfigsError,
    },
    #[fail(display = "failed to build stream: {}", err)]
    BuildStream { err: cpal::BuildStreamError },
}

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
    pub fn cpal_stream_config(&self) -> &cpal::SupportedStreamConfig {
        &self.cpal_stream_config
    }
}

impl<M> Shared<M> {
    fn play(&self) -> Result<(), cpal::PlayStreamError> {
        self.event_loop.play_stream(self.stream_id.clone())?;
        self.is_paused.store(false, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn pause(&self) -> Result<(), cpal::PauseStreamError> {
        self.is_paused.store(true, atomic::Ordering::Relaxed);
        self.event_loop.pause_stream(self.stream_id.clone())?;
        Ok(())
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
        let cpal_stream_config = self.cpal_stream_config.clone();
        Stream {
            update_tx,
            process_fn_tx,
            shared,
            cpal_stream_config,
        }
    }
}

impl<M> Drop for Shared<M> {
    fn drop(&mut self) {
        if self.is_playing() {
            self.pause().ok();
        }
        self.event_loop.destroy_stream(self.stream_id.clone());
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
fn matching_supported_configs(
    mut supported_stream_config_range: cpal::SupportedStreamConfigRange,
    sample_format: Option<cpal::SampleFormat>,
    channels: Option<usize>,
    sample_rate: Option<cpal::SampleRate>,
    buffer_size: Option<cpal::SupportedBufferSize>,
) -> Option<cpal::SupportedStreamConfig> {
    // Check for a matching sample format.
    if let Some(sample_format) = sample_format {
        if supported_stream_config_range.sample_format() != sample_format {
            return None;
        }
    }
    // Check for a matching number of channels.
    //
    // If there are more than enough channels, truncate the `SupportedFormat` to match.
    if let Some(channels) = channels {
        let supported_channels = supported_stream_config_range.channels() as usize;
        if supported_channels < channels {
            return None;
        }
    }
    // Check the sample rate.
    if let Some(sample_rate) = sample_rate {
        if supported_stream_config_range.min_sample_rate() > sample_rate
            || supported_stream_config_range.max_sample_rate() < sample_rate
        {
            return None;
        }
        let config = supported_stream_config_range.with_sample_rate(sample_rate);
        return Some(config);
    }

    // let config = cpal::SupportedStreamConfig {
    //     channels: channels,
    //     sample_rate: sample_rate,
    //     buffer_size: buffer_size,
    //     sample_format: sample_format,
    // };

    Some(supported_stream_config_range.with_max_sample_rate())
}

// Given some audio device find the supported stream config that best matches the given optional
// config parameters (specified by the user).
fn find_best_matching_config<F>(
    device: &cpal::Device,
    mut sample_format: Option<cpal::SampleFormat>,
    channels: Option<usize>,
    mut sample_rate: Option<cpal::SampleRate>,
    default_config: Option<cpal::SupportedStreamConfig>,
    buffer_size: Option<cpal::SupportedBufferSize>,
    supported_configs: F,
) -> Result<Option<cpal::SupportedStreamConfig>, cpal::SupportedStreamConfigsError>
where
    F: Fn(
        &cpal::Device,
    ) -> Result<Vec<cpal::SupportedStreamConfigRange>, cpal::SupportedStreamConfigsError>,
{
    loop {
        {
            // First, see if the default config satisfies the request.
            if let Some(ref config) = default_config {
                if sample_format == Some(config.sample_format())
                    && channels
                        .map(|ch| ch <= config.channels() as usize)
                        .unwrap_or(true)
                    && sample_rate == Some(config.sample_rate())
                    && buffer_size == Some(*config.buffer_size())
                {
                    return Ok(Some(config.clone()));
                }
            }

            // Otherwise search through all supported configs for compatible configs.
            let stream_configs = supported_configs(device)?.into_iter().filter_map(|config| {
                matching_supported_configs(
                    config,
                    sample_format,
                    channels,
                    sample_rate,
                    buffer_size,
                )
            });

            // Find the supported config with the most channels (this will always be the target
            // number of channels if some specific target number was specified as all other numbers
            // will have been filtered out already).
            if let Some(config) = stream_configs.max_by_key(|config| config.channels()) {
                return Ok(Some(config));
            }
        }

        // If there are no matching configs with the target sample_format, drop the requirement
        // and we'll do a conversion from the default config instead.
        let default_sample_format = default_config.as_ref().map(|f| f.sample_format());
        if sample_format.is_some() && sample_format != default_sample_format {
            sample_format = default_sample_format;
        // Otherwise if nannou's default target sample rate is set because the user didn't
        // specify a sample rate, try and fall back to a supported sample rate in case this was
        // the reason we could not find a supported format.
        } else if sample_rate == Some(cpal::SampleRate(DEFAULT_SAMPLE_RATE)) {
            let cpal_default_sample_rate =
                default_config.as_ref().map(|config| config.sample_rate());
            let nannou_default_sample_rate = Some(cpal::SampleRate(DEFAULT_SAMPLE_RATE));
            sample_rate = if cpal_default_sample_rate != nannou_default_sample_rate {
                cpal_default_sample_rate
            } else {
                None
            };
        } else {
            return Ok(None);
        }

        // TO DO probably need to also check against buffer size
    }
}
