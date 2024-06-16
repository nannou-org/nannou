use crate::{
    stream::{self, DefaultErrorFn, ErrorFn},
    Buffer, Device, Receiver, Stream,
};
use cpal::traits::{DeviceTrait, HostTrait};
use dasp_sample::{FromSample, Sample, ToSample};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// The function that will be called when a captured `Buffer` is ready to be read.
pub trait CaptureFn<M, S>: Fn(&mut M, &Buffer<S>) {}

/// The default capture function type used when unspecified.
pub type DefaultCaptureFn<M, S> = fn(&mut M, &Buffer<S>);

// The default capture function used when unspecified.
pub(crate) fn default_capture_fn<M, S>(_: &mut M, _: &Buffer<S>) {}

/// A type used for building an input stream.
pub struct Builder<M, FC, FE, S = f32> {
    pub builder: super::Builder<M, S>,
    pub capture: FC,
    pub error: FE,
}

/// The builder when first initialised.
pub type BuilderInit<M, S = f32> = Builder<M, DefaultCaptureFn<M, S>, DefaultErrorFn<M>, S>;

type InputDevices = cpal::InputDevices<cpal::Devices>;

/// An iterator yielding all available audio devices that support input streams.
pub struct Devices {
    pub(crate) devices: InputDevices,
}

impl<M, S, F> CaptureFn<M, S> for F where F: Fn(&mut M, &Buffer<S>) {}

impl<M, FC, FE, S> Builder<M, FC, FE, S> {
    /// Specify the capture function to use for updating the model in accordance with the captured
    /// input samples.
    pub fn capture<GC>(self, capture: GC) -> Builder<M, GC, FE, S> {
        let Builder { builder, error, .. } = self;
        Builder {
            builder,
            capture,
            error,
        }
    }

    /// Specify a function for handling stream errors.
    pub fn error<GE>(self, error: GE) -> Builder<M, FC, GE, S> {
        let Builder {
            builder, capture, ..
        } = self;
        Builder {
            builder,
            capture,
            error,
        }
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

    pub fn device_buffer_size(mut self, buffer_size: cpal::BufferSize) -> Self {
        self.builder.device_buffer_size = Some(buffer_size);
        self
    }

    pub fn build(self) -> std::result::Result<Stream<M>, super::BuildError>
    where
        S: 'static + Send + Sample + FromSample<u16> + FromSample<i16> + FromSample<f32>,
        M: 'static + Send,
        FC: 'static + CaptureFn<M, S> + Send,
        FE: 'static + ErrorFn<M> + Send,
    {
        let Builder {
            capture,
            error,
            builder:
                stream::Builder {
                    host,
                    model,
                    sample_rate,
                    channels,
                    frames_per_buffer,
                    device_buffer_size,
                    device,
                    ..
                },
        } = self;

        let device = match device {
            None => host
                .default_input_device()
                .ok_or(super::BuildError::DefaultDevice)?,
            Some(Device { device }) => device,
        };

        let desired = super::DesiredStreamConfig {
            sample_format: super::cpal_sample_format::<S>(),
            channels,
            sample_rate: sample_rate.map(cpal::SampleRate),
            device_buffer_size,
        };

        // Find the best matching config.
        let matching = super::find_best_matching_config(
            &device,
            desired,
            device.default_input_config().ok(),
            |device| device.supported_input_configs().map(|fs| fs.collect()),
        )?
        .expect("no matching supported audio input formats for the target device");
        let (update_tx, update_rx) = mpsc::channel();
        let model = Arc::new(Mutex::new(Some(model)));
        let model_render = model.clone();
        let model_error = model.clone();
        let num_channels = matching.config.channels as usize;
        let sample_rate = matching.config.sample_rate.0;
        let sample_format = matching.sample_format;
        let stream_config = matching.config;

        // A buffer for collecting model updates.
        let mut pending_updates: Vec<Box<dyn FnMut(&mut M) + 'static + Send>> = Vec::new();

        // Get the specified frames_per_buffer or fall back to a default.
        let frames_per_buffer = frames_per_buffer.unwrap_or(Buffer::<S>::DEFAULT_LEN_FRAMES);

        // A `Receiver` for converting audio delivered by the backend at varying buffer sizes into
        // buffers of a fixed size.
        let mut receiver = Receiver::new(frames_per_buffer, num_channels);

        // An intermediary buffer for converting cpal samples to the target sample
        // format.
        let mut samples = vec![S::EQUILIBRIUM; frames_per_buffer * num_channels];

        // The function used to process a buffer of samples.
        let capture_fn = move |data: &cpal::Data, _info: &cpal::InputCallbackInfo| {
            // Collect and process any pending updates.
            macro_rules! process_pending_updates {
                () => {
                    // Collect any pending updates.
                    pending_updates.extend(update_rx.try_iter());

                    // If there are some updates available, take the lock and apply them.
                    if !pending_updates.is_empty() {
                        if let Ok(mut guard) = model_render.lock() {
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
            samples.resize(data.len(), S::EQUILIBRIUM);

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

            match sample_format {
                cpal::SampleFormat::U16 => {
                    let input = data.as_slice::<u16>().expect("expected u16 data");
                    fill_input(&mut samples, input);
                }
                cpal::SampleFormat::I16 => {
                    let input = data.as_slice::<i16>().expect("expected i16 data");
                    fill_input(&mut samples, input);
                }
                cpal::SampleFormat::F32 => {
                    let input = data.as_slice::<f32>().expect("expected f32 data");
                    fill_input(&mut samples, input);
                }
            }

            if let Ok(mut guard) = model_render.lock() {
                let mut m = guard.take().unwrap();
                m = receiver.read_buffer(m, &capture, &samples, num_channels, sample_rate);
                *guard = Some(m);
            }

            process_pending_updates!();
        };

        // Wrap the user's error function.
        let err_fn = move |err| {
            if let Ok(mut guard) = model_error.lock() {
                if let Some(ref mut model) = *guard {
                    error(model, err);
                }
            }
        };

        let stream =
            device.build_input_stream_raw(&stream_config, sample_format, capture_fn, err_fn)?;

        let shared = Arc::new(super::Shared {
            stream,
            model,
            is_paused: AtomicBool::new(false),
        });

        let stream = Stream {
            shared,
            update_tx,
            cpal_config: stream_config,
        };
        Ok(stream)
    }
}

impl Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        self.devices.next().map(|device| Device { device })
    }
}
