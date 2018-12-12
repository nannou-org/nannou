use audio::cpal;
use audio::sample::{FromSample, Sample, ToSample};
use audio::stream;
use audio::{Buffer, Device, Receiver, Stream};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// The function that will be called when a captured `Buffer` is ready to be read.
pub trait CaptureFn<M, S>: Fn(&mut M, &Buffer<S>) {}
impl<M, S, F> CaptureFn<M, S> for F where F: Fn(&mut M, &Buffer<S>) {}

pub struct Builder<M, F, S = f32> {
    pub builder: super::Builder<M, S>,
    pub capture: F,
}

/// An iterator yielding all available audio devices that support input streams.
pub struct Devices {
    pub(crate) devices: cpal::InputDevices,
}

impl Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        self.devices.next().map(|device| Device { device })
    }
}

impl<M, F, S> Builder<M, F, S> {
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

    pub fn build(self) -> Result<Stream<M>, super::BuildError>
    where
        S: 'static + Send + Sample + FromSample<u16> + FromSample<i16> + FromSample<f32>,
        M: 'static + Send,
        F: 'static + CaptureFn<M, S> + Send,
    {
        let Builder {
            capture,
            builder:
                stream::Builder {
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
            None => cpal::default_input_device().ok_or(super::BuildError::DefaultDevice)?,
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
        )?.expect("no matching supported audio input formats for the target device");
        let stream_id = event_loop.build_input_stream(&device, &format)?;
        let (update_tx, update_rx) = mpsc::channel();
        let model = Arc::new(Mutex::new(Some(model)));
        let model_2 = model.clone();
        let num_channels = format.channels as usize;
        let sample_rate = format.sample_rate.0;

        // A buffer for collecting model updates.
        let mut pending_updates: Vec<Box<FnMut(&mut M) + 'static + Send>> = Vec::new();

        // Get the specified frames_per_buffer or fall back to a default.
        let frames_per_buffer = frames_per_buffer.unwrap_or(Buffer::<S>::DEFAULT_LEN_FRAMES);

        // A `Receiver` for converting audio delivered by the backend at varying buffer sizes into
        // buffers of a fixed size.
        let mut receiver = Receiver::new(frames_per_buffer, num_channels);

        // An intermediary buffer for converting cpal samples to the target sample
        // format.
        let mut samples = vec![S::equilibrium(); frames_per_buffer * num_channels];

        // The function used to process a buffer of samples.
        let proc_input = move |data: cpal::StreamData| {
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
                cpal::StreamData::Input { buffer } => buffer,
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
