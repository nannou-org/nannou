use crate::{stream, Buffer, Device, Requester, Stream, StreamError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp_sample::{Sample, ToSample};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// The buffer if it is ready for reading, or an error if something went wrong with the stream.
pub type Result<'a, S = f32> = std::result::Result<&'a mut Buffer<S>, StreamError>;

/// The function that will be called when a `Buffer` is ready to be rendered.
pub trait RenderFn<M, S>: Fn(&mut M, &mut Buffer<S>) {}
/// The function that will be called when a `StreamResult` is ready to be rendered.
pub trait RenderResultFn<M, S>: Fn(&mut M, Result<S>) {}

/// The default render function type used when unspecified.
pub type DefaultRenderFn<M, S> = fn(&mut M, &mut Buffer<S>);
/// The default render function type used when unspecified.
pub type DefaultRenderResultFn<M, S> = fn(&mut M, Result<S>);

// The default render function used when unspecified.
pub(crate) fn default_render_fn<M, S>(_: &mut M, _: &mut Buffer<S>) {}

/// Either a function for processing a stream result, or a function for processing the buffer
/// directly.
pub enum Render<A, B> {
    /// A function that works directly with the buffer.
    ///
    /// Any stream errors that occur will cause a `panic!`.
    BufferFn(A),
    /// A function that handles the stream result and in turn the inner buffer.
    ResultFn(B),
}

/// A type used for building an output stream.
pub struct Builder<M, FA, FB, S = f32> {
    pub builder: super::Builder<M, S>,
    pub render: Render<FA, FB>,
}

/// The builder when first initialised.
pub type BuilderInit<M, S = f32> =
    Builder<M, DefaultRenderFn<M, S>, DefaultRenderResultFn<M, S>, S>;

type OutputDevices = cpal::OutputDevices<cpal::Devices>;

/// An iterator yielding all available audio devices that support output streams.
pub struct Devices {
    pub(crate) devices: OutputDevices,
}

impl<M, S, F> RenderFn<M, S> for F where F: Fn(&mut M, &mut Buffer<S>) {}
impl<M, S, F> RenderResultFn<M, S> for F where F: Fn(&mut M, Result<S>) {}

impl<A, B> Render<A, B> {
    pub(crate) fn render<M, S>(&self, model: &mut M, result: Result<S>)
    where
        A: RenderFn<M, S>,
        B: RenderResultFn<M, S>,
    {
        match *self {
            Render::BufferFn(ref f) => {
                let buffer = match result {
                    Ok(b) => b,
                    Err(err) => {
                        panic!(
                            "An output stream error occurred: {}\nIf you wish to handle this \
                             error within your code, consider building your output stream with \
                             a `render_result` function rather than a `render` function.",
                            err,
                        );
                    }
                };
                f(model, buffer);
            }
            Render::ResultFn(ref f) => f(model, result),
        }
    }
}

impl<M, FA, FB, S> Builder<M, FA, FB, S> {
    /// Specify the render function to use for rendering the model to the buffer.
    ///
    /// If you wish to handle errors produced by the stream, you should use `render_result` instead
    /// and ensure your `render` function accepts a `output::Result<S>` rather than a `&mut
    /// Buffer<S>`.
    ///
    /// Please note that only one `render` or `render_result` function may be submitted. Only the
    /// last function submitted will be used.
    pub fn render<G>(self, render: G) -> Builder<M, G, DefaultRenderResultFn<M, S>, S> {
        let Builder { builder, .. } = self;
        let render = Render::BufferFn(render);
        Builder { render, builder }
    }

    /// Specify a function for processing render stream results.
    ///
    /// Please note that only one `render` or `render_result` function may be submitted. Only the
    /// last function submitted will be used.
    pub fn render_result<G>(self, render_result: G) -> Builder<M, DefaultRenderFn<M, S>, G, S> {
        let Builder { builder, .. } = self;
        let render = Render::ResultFn(render_result);
        Builder { render, builder }
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

    // TO DO add a buffer_size function

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
        S: 'static + Send + Sample + ToSample<u16> + ToSample<i16> + ToSample<f32>,
        M: 'static + Send,
        FA: 'static + RenderFn<M, S> + Send,
        FB: 'static + RenderResultFn<M, S> + Send,
    {
        let Builder {
            render,
            builder:
                stream::Builder {
                    host,
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
            None => host
                .default_output_device()
                .ok_or(super::BuildError::DefaultDevice)?,
            Some(Device { device }) => device,
        };

        // Find the best matching config.
        let config = super::find_best_matching_config(
            &device,
            sample_format,
            channels,
            sample_rate,
            device.default_output_config().ok(),
            |device| device.supported_output_configs().map(|fs| fs.collect()),
        )?
        .expect("no matching supported audio output formats for the target device");
        let stream_id = event_loop.build_output_stream(&device, &format)?;
        let (update_tx, update_rx) = mpsc::channel();
        let model = Arc::new(Mutex::new(Some(model)));
        let model_2 = model.clone();
        let num_channels = config.channels() as usize;
        let sample_rate = config.sample_rate().0;

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
        let proc_output = move |data: cpal::StreamDataResult| {
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

            // Retrieve the output buffer.
            let output = match data {
                Err(err) => {
                    if let Ok(mut guard) = model_2.lock() {
                        let mut m = guard.take().unwrap();
                        render.render(&mut m, Err(err));
                    }
                    return;
                }
                Ok(cpal::StreamData::Output { buffer }) => buffer,
                _ => unreachable!(),
            };

            samples.clear();
            samples.resize(output.len(), S::EQUILIBRIUM);

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

            match config.sample_format() {
                cpal::SampleFormat::F32 => {}
            }

            // Process the given buffer.
            match output {
                cpal::UnknownTypeOutputBuffer::U16(mut buffer) => {
                    fill_output(&mut buffer, &samples);
                }
                cpal::UnknownTypeOutputBuffer::I16(mut buffer) => {
                    fill_output(&mut buffer, &samples);
                }
                cpal::UnknownTypeOutputBuffer::F32(mut buffer) => {
                    fill_output(&mut buffer, &samples)
                }
            }

            process_pending_updates!();
        };

        // Send the buffer processing function to the event loop.
        process_fn_tx
            .send((stream_id.clone(), Box::new(proc_output)))
            .unwrap();

        let shared = Arc::new(super::Shared {
            model,
            stream_id,
            is_paused: AtomicBool::new(false),
        });

        let stream = Stream {
            shared,
            process_fn_tx,
            update_tx,
            cpal_stream_config: config,
        };
        Ok(stream)
    }
}

impl<M, S> Default for Render<DefaultRenderFn<M, S>, DefaultRenderResultFn<M, S>> {
    fn default() -> Self {
        Render::BufferFn(default_render_fn)
    }
}

impl Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        self.devices.next().map(|device| Device { device })
    }
}
