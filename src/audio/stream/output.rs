use audio::Buffer;
use audio::Requester;
use audio::cpal;
use audio::sample::{Sample, ToSample};
use std;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

// TODO: For some reason using the render function in the cpal event loop requires it to be
// Sync, but I don't think this should really be necessary.
pub trait RenderFn<M, S>: Fn(M, Buffer<S>) -> (M, Buffer<S>) {}
impl<M, S, F> RenderFn<M, S> for F where F: Fn(M, Buffer<S>) -> (M, Buffer<S>) {}

/// A handle around an output audio stream.
pub struct Output<M> {
    /// The user's audio model
    model: Arc<Mutex<Option<M>>>,
    /// A unique ID associated with this stream's "voice" on the cpal EventLoop.
    voice_id: cpal::VoiceId,
    /// A handle to the CPAL audio event loop.
    event_loop: Arc<cpal::EventLoop>,
    /// A channel for sending model updates to the audio thread.
    update_tx: mpsc::Sender<Box<FnMut(&mut M) + 'static + Send>>,
    /// Whether or not the stream is currently paused.
    is_paused: bool,
}

// Manually implement `Clone` to avoid requiring that `M: Clone`.
impl<M> Clone for Output<M> {
    fn clone(&self) -> Self {
        Output {
            model: self.model.clone(),
            voice_id: self.voice_id.clone(),
            event_loop: self.event_loop.clone(),
            update_tx: self.update_tx.clone(),
            is_paused: self.is_paused,
        }
    }
}

pub struct Builder<M, F, S=f32> {
    pub(crate) event_loop: Arc<cpal::EventLoop>,
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
        where S: Sample + ToSample<u16> + ToSample<i16> + ToSample<f32>,
              M: 'static + Send,
              F: 'static + RenderFn<M, S> + Send,
    {
        let Builder {
            event_loop,
            model,
            render,
            sample_rate,
            channels,
            frames_per_buffer,
            device,
            ..
        } = self;

        let endpoint = match device {
            None => cpal::default_endpoint().ok_or(BuildError::DefaultDevice)?,
            Some(Device { endpoint }) => endpoint,
        };

        let supported_format = endpoint
            .supported_formats()?
            .next()
            .expect("Failed to get any output device stream formats");
        let min_sample_rate = supported_format.min_samples_rate;
        let max_sample_rate = supported_format.max_samples_rate;
        let mut format = supported_format.with_max_samples_rate();

        if let Some(ch) = channels {
            format.channels.resize(ch, cpal::ChannelPosition::FrontLeft);
        }
        if let Some(sr) = sample_rate {
            format.samples_rate = cpal::SamplesRate(sr);
        } else {
            let default = cpal::SamplesRate(super::DEFAULT_SAMPLE_RATE);
            if default <= max_sample_rate && default >= min_sample_rate {
                format.samples_rate = default;
            }
        }

        let voice_id = event_loop.build_voice(&endpoint, &format)?;
        let (update_tx, update_rx) = mpsc::channel();
        let model = Arc::new(Mutex::new(Some(model)));
        let model_2 = model.clone();
        let event_loop_2 = event_loop.clone();
        let num_channels = format.channels.len();
        let sample_rate = format.samples_rate.0;

        std::thread::Builder::new()
            .name(format!("cpal audio output stream: {}", endpoint.name()))
            .spawn(move || {
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

                // Run the loop, in turn blocking the thread.
                event_loop_2.run(move |_voice_id, mut output| {

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
                })
            })
            .expect("Failed to spawn thread for cpal audio output stream");

        let output = Output {
            model,
            voice_id,
            event_loop,
            update_tx,
            is_paused: true,
        };
        Ok(output)
    }
}

impl<M> Output<M> {
    /// Command the audio device to start playing this stream.
    ///
    /// Calling this will activate rendering, in turn calling the given audio render function.
    ///
    /// Has no effect if the stream is already playing.
    pub fn play(&mut self) {
        self.event_loop.play(self.voice_id.clone());
        self.is_paused = false;
    }

    /// Command the audio device to stop playback.
    ///
    /// Calling this will pause rendering, in turn .
    ///
    /// Has no effect is the voice was already paused.
    pub fn pause(&mut self) {
        self.is_paused = true;
        self.event_loop.pause(self.voice_id.clone());
    }

    /// Whether or not the stream is currently playing.
    pub fn is_playing(&self) -> bool {
        !self.is_paused
    }

    /// Whether or not the stream is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_paused
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
        if self.is_paused {
            if let Ok(mut guard) = self.model.lock() {
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

impl<M> Drop for Output<M> {
    fn drop(&mut self) {
        if self.is_playing() {
            self.pause();
        }
        self.event_loop.destroy_voice(self.voice_id.clone());
    }
}
