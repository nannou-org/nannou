use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use crate::{stream, Buffer, Device, Receiver, Stream};
use sample::{FromSample, Sample, ToSample};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// The buffer if it is ready for reading, or an error if something went wrong with the stream.
pub type Result<'a, S = f32> = std::result::Result<&'a Buffer<S>, cpal::StreamError>;

/// The function that will be called when a captured `Buffer` is ready to be read.
pub trait CaptureFn<M, S>: Fn(&mut M, &Buffer<S>) {}
/// The function that will be called when a stream result is ready to be handled.
pub trait CaptureResultFn<M, S>: Fn(&mut M, Result<S>) {}

/// The default capture function type used when unspecified.
pub type DefaultCaptureFn<M, S> = fn(&mut M, &Buffer<S>);
/// The default capture function type used when unspecified.
pub type DefaultCaptureResultFn<M, S> = fn(&mut M, Result<S>);

// The default render function used when unspecified.
pub(crate) fn default_capture_fn<M, S>(_: &mut M, _: &Buffer<S>) {}

/// Either a function for processing a stream result, or a function for processing the buffer
/// directly.
pub enum Capture<A, B> {
    /// A function that works directly with the buffer.
    ///
    /// Any stream errors that occur will cause a `panic!`.
    BufferFn(A),
    /// A function that handles the stream result and in turn the inner buffer.
    ResultFn(B),
}

/// A type used for building an input stream.
pub struct Builder<M, FA, FB, S = f32> {
    pub builder: super::Builder<M, S>,
    pub capture: Capture<FA, FB>,
}

/// The builder when first initialised.
pub type BuilderInit<M, S = f32> =
    Builder<M, DefaultCaptureFn<M, S>, DefaultCaptureResultFn<M, S>, S>;

type InputDevices = cpal::InputDevices<cpal::platform::Devices>;

/// An iterator yielding all available audio devices that support input streams.
pub struct Devices {
    pub(crate) devices: InputDevices,
}

impl<M, S, F> CaptureFn<M, S> for F where F: Fn(&mut M, &Buffer<S>) {}
impl<M, S, F> CaptureResultFn<M, S> for F where F: Fn(&mut M, Result<S>) {}

impl<A, B> Capture<A, B> {
    pub(crate) fn capture<M, S>(&self, model: &mut M, result: Result<S>)
    where
        A: CaptureFn<M, S>,
        B: CaptureResultFn<M, S>,
    {
        match *self {
            Capture::BufferFn(ref f) => {
                let buffer = match result {
                    Ok(b) => b,
                    Err(err) => {
                        panic!(
                            "An input stream error occurred: {}\nIf you wish to handle this \
                            error within your code, consider building your input stream with \
                            a `capture_result` function rather than a `capture` function.",
                            err,
                        );
                    }
                };
                f(model, buffer);
            }
            Capture::ResultFn(ref f) => f(model, result),
        }
    }
}

impl<M, FA, FB, S> Builder<M, FA, FB, S> {
    /// Specify the capture function to use for updating the model in accordance with the captured
    /// input samples.
    ///
    /// If you wish to handle errors produced by the stream, you should use `capture_result`
    /// instead and ensure your `capture` function accepts a `input::Result<S>` rather than a
    /// `&mut Buffer<S>`.
    ///
    /// Please note that only one `capture` or `capture_result` function may be submitted. Only the
    /// last function submitted will be used.
    pub fn capture<G>(self, capture: G) -> Builder<M, G, DefaultCaptureResultFn<M, S>, S> {
        let Builder { builder, .. } = self;
        let capture = Capture::BufferFn(capture);
        Builder { capture, builder }
    }

    /// Specify a function for processing capture stream results.
    ///
    /// Please note that only one `capture` or `capture_result` function may be submitted. Only the
    /// last function submitted will be used.
    pub fn capture_result<G>(self, capture_result: G) -> Builder<M, DefaultCaptureFn<M, S>, G, S> {
        let Builder { builder, .. } = self;
        let capture = Capture::ResultFn(capture_result);
        Builder { capture, builder }
    }

    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        assert!(sample_rate > 0);
        self.builder.sample_rate = Some(sample_rate);
        self
    }

    pub fn channels(mut self, channels: usize) -> Self {
        assert!(channels > 0);
        self.builder.channels = Some(channels);
        self
    }

    pub fn device(mut self, device: Device) -> Self {
        self.builder.device = Some(device);
        self
    }

    pub fn frames_per_buffer(mut self, frames_per_buffer: usize) -> Self {
        assert!(frames_per_buffer > 0);
        self.builder.frames_per_buffer = Some(frames_per_buffer);
        self
    }

    pub fn build(self) -> std::result::Result<Stream<M>, super::BuildError>
    where
        S: 'static + Send + Sample + FromSample<u16> + FromSample<i16> + FromSample<f32>,
        M: 'static + Send,
        FA: 'static + CaptureFn<M, S> + Send,
        FB: 'static + CaptureResultFn<M, S> + Send,
    {
        let Builder {
            capture,
            builder:
                stream::Builder {
                    host,
                    event_loop,
                    process_fn_tx,
                    model,
                    sample_rate,
                    channels,
                    frames_per_buffer,
                    device,
                    ..
                },
        } = self;

        let sample_rate = sample_rate
            .map(|sr| cpal::SampleRate(sr))
            .or(Some(cpal::SampleRate(super::DEFAULT_SAMPLE_RATE)));
        let sample_format = super::cpal_sample_format::<S>();

        let device = match device {
            None => host.default_input_device().ok_or(super::BuildError::DefaultDevice)?,
            Some(Device { device }) => device,
        };

        // Find the best matching format.
        let format = super::find_best_matching_format(
            &device,
            sample_format,
            channels,
            sample_rate,
            device.default_input_format().ok(),
            |device| device.supported_input_formats().map(|fs| fs.collect()),
        )?
        .expect("no matching supported audio input formats for the target device");
        let stream_id = event_loop.build_input_stream(&device, &format)?;
        let (update_tx, update_rx) = mpsc::channel();
        let model = Arc::new(Mutex::new(Some(model)));
        let model_2 = model.clone();
        let num_channels = format.channels as usize;
        let sample_rate = format.sample_rate.0;

        // A buffer for collecting model updates.
        let mut pending_updates: Vec<Box<dyn FnMut(&mut M) + 'static + Send>> = Vec::new();

        // Get the specified frames_per_buffer or fall back to a default.
        let frames_per_buffer = frames_per_buffer.unwrap_or(Buffer::<S>::DEFAULT_LEN_FRAMES);

        // A `Receiver` for converting audio delivered by the backend at varying buffer sizes into
        // buffers of a fixed size.
        let mut receiver = Receiver::new(frames_per_buffer, num_channels);

        // An intermediary buffer for converting cpal samples to the target sample
        // format.
        let mut samples = vec![S::equilibrium(); frames_per_buffer * num_channels];

        // The function used to process a buffer of samples.
        let proc_input = move |data: cpal::StreamDataResult| {
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

            // Retrieve the input buffer.
            let input = match data {
                Err(err) => {
                    if let Ok(mut guard) = model_2.lock() {
                        let mut m = guard.take().unwrap();
                        capture.capture(&mut m, Err(err));
                    }
                    return;
                }
                Ok(cpal::StreamData::Input { buffer }) => buffer,
                _ => unreachable!(),
            };

            samples.clear();
            samples.resize(input.len(), S::equilibrium());

            // A function to simplify reading from the unknown buffer type.
            fn fill_input<I, S>(input: &mut [I], buffer: &[S])
            where
                I: Sample,
                S: Sample + ToSample<I>,
            {
                for (in_sample, sample) in input.iter_mut().zip(buffer) {
                    *in_sample = sample.to_sample();
                }
            }

            match input {
                cpal::UnknownTypeInputBuffer::U16(buffer) => {
                    fill_input(&mut samples, &buffer);
                }
                cpal::UnknownTypeInputBuffer::I16(buffer) => {
                    fill_input(&mut samples, &buffer);
                }
                cpal::UnknownTypeInputBuffer::F32(buffer) => {
                    fill_input(&mut samples, &buffer);
                }
            }

            if let Ok(mut guard) = model_2.lock() {
                let mut m = guard.take().unwrap();
                m = receiver.read_buffer(m, &capture, &samples, num_channels, sample_rate);
                *guard = Some(m);
            }

            process_pending_updates!();
        };

        // Send the buffer processing function to the event loop.
        process_fn_tx
            .send((stream_id.clone(), Box::new(proc_input)))
            .unwrap();

        let shared = Arc::new(super::Shared {
            model,
            stream_id,
            event_loop,
            is_paused: AtomicBool::new(false),
        });

        let stream = Stream {
            shared,
            process_fn_tx,
            update_tx,
            cpal_format: format,
        };
        Ok(stream)
    }
}

impl<M, S> Default for Capture<DefaultCaptureFn<M, S>, DefaultCaptureResultFn<M, S>> {
    fn default() -> Self {
        Capture::BufferFn(default_capture_fn)
    }
}

impl Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        self.devices.next().map(|device| Device { device })
    }
}
