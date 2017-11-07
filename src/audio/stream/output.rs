use audio::Buffer;
use audio::Requester;
use audio::cpal;
use audio::sample::{Sample, ToSample};
use std;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{self, AtomicBool};
use std::sync::mpsc;

// TODO: For some reason using the render function in the cpal event loop requires it to be
// Sync, but I don't think this should really be necessary.
pub trait RenderFn<M, S>: Fn(M, Buffer<S>) -> (M, Buffer<S>) {}
impl<M, S, F> RenderFn<M, S> for F where F: Fn(M, Buffer<S>) -> (M, Buffer<S>) {}

/// A clone-able handle around an output audio stream.
pub struct Output<M> {
    /// A channel for sending model updates to the audio thread.
    update_tx: mpsc::Sender<Box<FnMut(&mut M) + 'static + Send>>,
    /// A channel used for sending through a new buffer processing callback to the event loop.
    process_fn_tx: mpsc::Sender<ProcessFnMsg>,
    /// Data shared between each `Output` handle to a single stream.
    shared: Arc<Shared<M>>,
}

impl<M> Clone for Output<M> {
    fn clone(&self) -> Self {
        let update_tx = self.update_tx.clone();
        let process_fn_tx = self.process_fn_tx.clone();
        let shared = self.shared.clone();
        Output { update_tx, process_fn_tx, shared }
    }
}

// Data shared between each `Output` handle to a single stream.
struct Shared<M> {
    /// The user's audio model
    model: Arc<Mutex<Option<M>>>,
    /// A unique ID associated with this stream's "voice" on the cpal EventLoop.
    voice_id: cpal::VoiceId,
    /// A handle to the CPAL audio event loop.
    event_loop: Arc<cpal::EventLoop>,
    /// Whether or not the stream is currently paused.
    is_paused: AtomicBool,
}

pub(crate) type ProcessFn = FnMut(cpal::UnknownTypeBuffer) + 'static + Send;
pub(crate) type ProcessFnMsg = (cpal::VoiceId, Box<ProcessFn>);

pub struct Builder<M, F, S=f32> {
    pub(crate) event_loop: Arc<cpal::EventLoop>,
    pub(crate) process_fn_tx: mpsc::Sender<ProcessFnMsg>,
    pub model: M,
    pub render: F,
    pub sample_rate: Option<u32>,
    pub channels: Option<usize>,
    pub frames_per_buffer: Option<usize>,
    pub device: Option<Device>,
    pub(crate) sample_format: PhantomData<S>,
}

/// An iterator yielding a `audio::stream::DeviceId` for each output device.
pub struct Devices {
    pub(crate) endpoints: cpal::EndpointsIterator,
}

/// A device that can be used to spawn an output audio stream.
pub struct Device {
    pub(crate) endpoint: cpal::Endpoint,
}

// State that is updated and run on the `cpal::EventLoop::run` thread.
pub(crate) struct LoopContext {
    // A channel for receiving callback functions for newly spawned voices.
    process_fn_rx: mpsc::Receiver<ProcessFnMsg>,
    // A map from VoiceIds to their associated buffer processing functions.
    process_fns: HashMap<cpal::VoiceId, Box<ProcessFn>>,
}

impl LoopContext {
    /// Create a new loop context.
    pub fn new(process_fn_rx: mpsc::Receiver<ProcessFnMsg>) -> Self {
        let process_fns = HashMap::new();
        LoopContext { process_fn_rx, process_fns }
    }

    /// Process the given buffer with the voice at the given ID.
    pub fn process(&mut self, voice_id: cpal::VoiceId, mut buffer: cpal::UnknownTypeBuffer) {
        // Collect any pending voice process fns.
        for (voice_id, proc_fn) in self.process_fn_rx.try_iter() {
            self.process_fns.insert(voice_id, proc_fn);
        }

        // Process the buffer using the voice at the given ID.
        if let Some(proc_fn) = self.process_fns.get_mut(&voice_id) {
            proc_fn(buffer);
        // If there is not yet a buffer processing function, just silence the buffer.
        } else {
            fn silence<S: Sample>(slice: &mut [S]) {
                for sample in slice {
                    *sample = S::equilibrium();
                }
            }
            match buffer {
                cpal::UnknownTypeBuffer::U16(ref mut buffer) => silence(buffer),
                cpal::UnknownTypeBuffer::I16(ref mut buffer) => silence(buffer),
                cpal::UnknownTypeBuffer::F32(ref mut buffer) => silence(buffer),
            }
        }
    }
}

impl Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        self.endpoints.next().map(|endpoint| Device { endpoint })
    }
}

impl Device {
    /// An iterator yielding all PCM stream formats supported by the device.
    pub fn supported_formats(&self)
        -> Result<cpal::SupportedFormatsIterator, cpal::FormatsEnumerationError>
    {
        self.endpoint.supported_formats()
    }

    /// The name of the device.
    pub fn name(&self) -> String {
        self.endpoint.name()
    }
}

#[derive(Debug)]
pub enum BuildError {
    DefaultDevice,
    FormatEnumeration(cpal::FormatsEnumerationError),
    Creation(cpal::CreationError),
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
            BuildError::DefaultDevice => "Failed to get default output device",
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

impl<M, F, S> Builder<M, F, S> {
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        assert!(sample_rate > 0);
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn channels(mut self, channels: usize) -> Self {
        assert!(channels > 0);
        self.channels = Some(channels);
        self
    }

    pub fn device(mut self, device: Device) -> Self {
        self.device = Some(device);
        self
    }

    pub fn frames_per_buffer(mut self, frames_per_buffer: usize) -> Self {
        assert!(frames_per_buffer > 0);
        self.frames_per_buffer = Some(frames_per_buffer);
        self
    }

    pub fn build(self) -> Result<Output<M>, BuildError>
        where S: 'static + Send + Sample + ToSample<u16> + ToSample<i16> + ToSample<f32>,
              M: 'static + Send,
              F: 'static + RenderFn<M, S> + Send,
    {
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

        // If the target params match the given `SupportedFormat`, returns the matching format.
        fn matching_format(
            mut supported_format: cpal::SupportedFormat,
            sample_format: Option<cpal::SampleFormat>,
            channels: Option<usize>,
            sample_rate: Option<cpal::SamplesRate>,
        ) -> Option<cpal::Format>
        {
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
                if supported_format.channels.len() < channels {
                    return None;
                } else if supported_format.channels.len() > channels {
                    supported_format.channels.truncate(channels);
                }
            }
            // Check the sample rate.
            if let Some(sample_rate) = sample_rate {
                if supported_format.min_samples_rate > sample_rate
                || supported_format.max_samples_rate < sample_rate
                {
                    return None;
                }
                let mut format = supported_format.with_max_samples_rate();
                format.samples_rate = sample_rate;
                return Some(format);
            }
            Some(supported_format.with_max_samples_rate())
        }

        let Builder {
            event_loop,
            process_fn_tx,
            model,
            render,
            sample_rate,
            channels,
            frames_per_buffer,
            device,
            ..
        } = self;

        let mut sample_rate = sample_rate
            .map(|sr| cpal::SamplesRate(sr))
            .or(Some(cpal::SamplesRate(super::DEFAULT_SAMPLE_RATE)));
        let mut sample_format = cpal_sample_format::<S>();

        let endpoint = match device {
            None => cpal::default_endpoint().ok_or(BuildError::DefaultDevice)?,
            Some(Device { endpoint }) => endpoint,
        };

        // Find the best matching format.
        let format = 'find_format: loop {
            {
                let mut sample_formats = endpoint
                    .supported_formats()?
                    .filter_map(|fmt| matching_format(fmt, sample_format, channels, sample_rate));

                // Find the supported format with the most channels (this will always be the target
                // number of channels if some specific target number was specified as all other numbers
                // will have been filtered out already).
                if let Some(first) = sample_formats.next() {
                    let format = sample_formats.fold(first, |max, fmt| {
                        if fmt.channels.len() > max.channels.len() { fmt } else { max }
                    });
                    break 'find_format format;
                }
            }

            // If there are no matching formats with the target sample_format, drop the requirement
            // and we'll do a conversion to it instead.
            if sample_format.is_some() {
                sample_format = None;
            // Otherwise if nannou's default target sample rate is set because the user didn't
            // specify a sample rate, try and fall back to a supported sample rate in case this was
            // the reason we could not find a supported sample rate.
            } else if sample_rate == Some(cpal::SamplesRate(super::DEFAULT_SAMPLE_RATE)) {
                sample_rate = None;
            } else {
                panic!("no matching supported audio output formats for the target device");
            }
        };

        let voice_id = event_loop.build_voice(&endpoint, &format)?;
        let (update_tx, update_rx) = mpsc::channel();
        let model = Arc::new(Mutex::new(Some(model)));
        let model_2 = model.clone();
        let num_channels = format.channels.len();
        let sample_rate = format.samples_rate.0;

        // A buffer for collecting model updates.
        let mut pending_updates: Vec<Box<FnMut(&mut M) + 'static + Send>> = Vec::new();

        // Get the specified frames_per_buffer or fall back to a default.
        let frames_per_buffer = frames_per_buffer.unwrap_or(Buffer::<S>::DEFAULT_LEN_FRAMES);

        // An audio requester which requests frames from the model+render pair with a
        // specific buffer size, regardless of the buffer size requested by the OS.
        let mut requester = Requester::new(frames_per_buffer, num_channels);

        // An intermediary buffer for converting cpal samples to the target sample
        // format.
        let mut samples = vec![S::equilibrium(); frames_per_buffer * num_channels];

        // The function used to process a buffer of samples.
        let proc_output = move |mut output: cpal::UnknownTypeBuffer| {

            // Collect and process any pending updates.
            macro_rules! process_pending_updates {
                () => {
                    // Collect any pending updates.
                    pending_updates.extend(update_rx.try_iter());

                    // If there are some updates available, take the lock and apply them.
                    if !pending_updates.is_empty() {
                        if let Ok(mut guard) = model_2.lock() {
                            let mut model = guard.take().unwrap();
                            for mut update in pending_updates.drain(..) {
                                update(&mut model);
                            }
                            *guard = Some(model);
                        }
                    }
                };
            }

            process_pending_updates!();

            samples.clear();
            samples.resize(output.len(), S::equilibrium());

            if let Ok(mut guard) = model_2.lock() {
                let mut m = guard.take().unwrap();
                m = requester.fill_buffer(m, &render, &mut samples, num_channels, sample_rate);
                *guard = Some(m);
            }

            // A function to simplify filling the unknown buffer type.
            fn fill_output<O, S>(output: &mut [O], buffer: &[S])
            where
                O: Sample,
                S: Sample + ToSample<O>,
            {
                for (out_sample, sample) in output.iter_mut().zip(buffer) {
                    *out_sample = sample.to_sample();
                }
            }

            // Process the given buffer.
            match output {
                cpal::UnknownTypeBuffer::U16(ref mut buffer) => fill_output(buffer, &samples),
                cpal::UnknownTypeBuffer::I16(ref mut buffer) => fill_output(buffer, &samples),
                cpal::UnknownTypeBuffer::F32(ref mut buffer) => fill_output(buffer, &samples),
            }

            process_pending_updates!();
        };

        // Send the buffer processing function to the event loop.
        process_fn_tx.send((voice_id.clone(), Box::new(proc_output))).unwrap();

        let shared = Arc::new(Shared {
            model,
            voice_id,
            event_loop,
            is_paused: AtomicBool::new(false),
        });

        let output = Output {
            shared,
            process_fn_tx,
            update_tx,
        };
        Ok(output)
    }
}

impl<M> Shared<M> {
    fn play(&self) {
        self.event_loop.play(self.voice_id.clone());
        self.is_paused.store(false, atomic::Ordering::Relaxed);
    }

    fn pause(&self) {
        self.is_paused.store(true, atomic::Ordering::Relaxed);
        self.event_loop.pause(self.voice_id.clone());
    }

    fn is_playing(&self) -> bool {
        !self.is_paused.load(atomic::Ordering::Relaxed)
    }

    fn is_paused(&self) -> bool {
        self.is_paused.load(atomic::Ordering::Relaxed)
    }
}

impl<M> Output<M> {
    /// Command the audio device to start playing this stream.
    ///
    /// Calling this will activate rendering, in turn calling the given audio render function.
    ///
    /// Has no effect if the stream is already playing.
    pub fn play(&self) {
        self.shared.play()
    }

    /// Command the audio device to stop playback.
    ///
    /// Calling this will pause rendering, in turn .
    ///
    /// Has no effect is the voice was already paused.
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
    pub fn send<F>(&self, update: F)
        -> Result<(), mpsc::SendError<Box<FnMut(&mut M) + Send + 'static>>>
        where F: FnOnce(&mut M) + Send + 'static,
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
}

impl<M> Drop for Shared<M> {
    fn drop(&mut self) {
        if self.is_playing() {
            self.pause();
        }
        self.event_loop.destroy_voice(self.voice_id.clone());
    }
}
