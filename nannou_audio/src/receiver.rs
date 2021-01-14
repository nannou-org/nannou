use crate::{stream, Buffer};
use dasp_sample::Sample;
use std;

/// A `Receiver` for converting audio delivered by the backend at varying buffer sizes into buffers
/// of a fixed size.
///
/// The `Receiver` works by calling `fill_buffer` with the requested buffer and sample rate from
/// the audio backend each time the callback is invoked.
pub struct Receiver<S> {
    samples: Vec<S>,
    num_frames: usize,
    num_channels: usize,
}

impl<S> Receiver<S>
where
    S: Sample,
{
    /// Construct a `Receiver`.
    ///
    /// Both `num_frames` and `num_channels` must be greater than `0`.
    pub fn new(num_frames: usize, num_channels: usize) -> Self {
        // We can't make any progress filling buffers of `0` frames.
        assert!(num_frames > 0);
        assert!(num_channels > 0);
        let num_samples = num_frames + num_channels;
        Receiver {
            samples: Vec::with_capacity(num_samples),
            num_frames: num_frames,
            num_channels: num_channels,
        }
    }

    /// Deliver samples from `input` to the given capture function in chunks of size `frames *
    /// channels`.
    ///
    /// **Panic!**s under any of the following conditions:
    ///
    /// - `sample_rate` is not greater than `0`.
    /// - The number of `channels` is different to that with which the receiver was initialised.
    /// - The final input buffer frame does not contain a sample for every channel.
    pub fn read_buffer<M, FC>(
        &mut self,
        mut model: M,
        capture: &FC,
        input: &[S],
        channels: usize,
        sample_rate: u32,
    ) -> M
    where
        FC: stream::input::CaptureFn<M, S>,
    {
        let Receiver {
            ref mut samples,
            num_frames,
            num_channels,
        } = *self;

        // Ensure that the input length makes sense given the number of channels.
        assert_eq!(
            input.len() % channels,
            0,
            "the input length must be a multiple of the number of channels"
        );
        // Ensure that the number of channels has not changed.
        assert_eq!(
            channels, num_channels,
            "the number of channels differs to that with which `Receiver` was initialised"
        );

        // An iterator yielding the input samples.
        let mut input_samples = input.iter().cloned();

        // The number of samples pass to the capture function at a time.
        let num_samples = num_frames * channels;

        // Append the given input and deliver it to the capture fn until we run out.
        loop {
            let num_to_take = num_samples - samples.len();
            samples.extend(input_samples.by_ref().take(num_to_take));

            // If there weren't enough samples remaining in the input for a full buffer we're done.
            if samples.len() < num_samples {
                break;
            }

            // Capture the input data and update the model.
            let interleaved_samples = std::mem::replace(samples, Vec::new()).into_boxed_slice();
            let buffer = Buffer {
                interleaved_samples,
                channels,
                sample_rate,
            };
            capture(&mut model, &buffer);
            std::mem::swap(samples, &mut buffer.interleaved_samples.into_vec());
            samples.clear();
        }

        model
    }
}
