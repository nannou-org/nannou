use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait};
use dasp_sample::{Sample, ToSample};

use crate::{
    stream::{self, DefaultErrorFn, ErrorFn},
    Buffer, Device, Requester, Stream,
};

/// The function that will be called when a `Buffer` is ready to be rendered.
pub trait RenderFn<M, S>: Fn(&mut M, &mut Buffer<S>) {}

/// The default render function type used when unspecified.
pub type DefaultRenderFn<M, S> = fn(&mut M, &mut Buffer<S>);

// The default render function used when unspecified.
pub(crate) fn default_render_fn<M, S>(_: &mut M, _: &mut Buffer<S>) {}

/// A type used for building an output stream.
pub struct Builder<M, FR, FE, S = f32> {
    pub builder: super::Builder<M, S>,
    pub render: FR,
    pub error: FE,
}

/// The builder when first initialised.
pub type BuilderInit<M, S = f32> = Builder<M, DefaultRenderFn<M, S>, DefaultErrorFn<M>, S>;

type OutputDevices = cpal::OutputDevices<cpal::Devices>;

/// An iterator yielding all available audio devices that support output streams.
pub struct Devices {
    pub(crate) devices: OutputDevices,
}

impl<M, S, F> RenderFn<M, S> for F where F: Fn(&mut M, &mut Buffer<S>) {}

impl<M, FR, FE, S> Builder<M, FR, FE, S> {
    /// Specify the render function to use for rendering the model to the buffer.
    pub fn render<GR>(self, render: GR) -> Builder<M, GR, FE, S> {
        let Builder { builder, error, .. } = self;
        Builder {
            builder,
            render,
            error,
        }
    }

    /// Specify a function for processing stream errors.
    pub fn error<GE>(self, error: GE) -> Builder<M, FR, GE, S> {
        let Builder {
            builder, render, ..
        } = self;
        Builder {
            builder,
            render,
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
        S: 'static + Send + Sample + ToSample<u16> + ToSample<i16> + ToSample<f32>,
        M: 'static + Send,
        FR: 'static + RenderFn<M, S> + Send,
        FE: 'static + ErrorFn<M> + Send,
    {
        let Builder {
            render,
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
                .default_output_device()
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
            device.default_output_config().ok(),
            |device| device.supported_output_configs().map(|fs| fs.collect()),
        )?
        .expect("no matching supported audio output formats for the target device");
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

        // An audio requester which requests frames from the model+render pair with a
        // specific buffer size, regardless of the buffer size requested by the OS.
        let mut requester = Requester::new(frames_per_buffer, num_channels);

        // An intermediary buffer for converting cpal samples to the target sample
        // format.
        let mut samples = vec![S::EQUILIBRIUM; frames_per_buffer * num_channels];

        // The function used to process a buffer of samples.
        // TODO: We should notify the user of `OutputCallbackInfo`.
        let render_fn = move |data: &mut cpal::Data, _info: &cpal::OutputCallbackInfo| {
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

            if let Ok(mut guard) = model_render.lock() {
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
            match sample_format {
                cpal::SampleFormat::U16 => {
                    let output = data.as_slice_mut::<u16>().expect("expected u16 data");
                    fill_output(output, &samples);
                }
                cpal::SampleFormat::I16 => {
                    let output = data.as_slice_mut::<i16>().expect("expected i16 data");
                    fill_output(output, &samples);
                }
                cpal::SampleFormat::F32 => {
                    let output = data.as_slice_mut::<f32>().expect("expected f32 data");
                    fill_output(output, &samples);
                }
            }
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
            device.build_output_stream_raw(&stream_config, sample_format, render_fn, err_fn)?;

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
