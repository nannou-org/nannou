use audio;
use audio::sample::Sample;
use std;

/// A `sound::Requester` for converting backend audio requests into requests for buffers of a fixed
/// size called from a separate thread.
///
/// The `Requester` works by calling `fill_buffer` with the requested buffer and sample rate from
/// the audio backend each time the callback is invoked.
pub struct Requester<S> {
    samples: Vec<S>,
    num_frames: usize,
    // `Some` if part of `frames` has not yet been written to output.
    pending_range: Option<std::ops::Range<usize>>,
}

impl<S> Requester<S>
where
    S: Sample,
{
    /// Construct a new `sound::Requester`.
    ///
    /// `num_frames` must be greater than `0`.
    pub fn new(num_frames: usize, num_channels: usize) -> Self {
        // We can't make any progress filling buffers of `0` frames.
        assert!(num_frames > 0);
        let num_samples = num_frames + num_channels;
        Requester {
            samples: vec![S::equilibrium(); num_samples],
            num_frames: num_frames,
            pending_range: None,
        }
    }

    /// Fill the given `output` buffer with samples requested from the model.
    ///
    /// `Panic!`s if `sample_rate` is not greater than `0` or if the output buffer's length is not
    /// a multiple of the given number of channels.
    pub fn fill_buffer<M, F>(
        &mut self,
        mut model: M,
        render: F,
        output: &mut [S],
        channels: usize,
        sample_rate: u32,
    ) -> M
    where
        F: audio::stream::output::RenderFn<M, S>,
    {
        let Requester {
            ref mut samples,
            num_frames,
            ref mut pending_range,
        } = *self;

        // Ensure that the buffer length makes sense given the number of channels.
        assert!(output.len() % channels == 0);

        // Determine the number of samples in the buffer.
        let num_samples = num_frames * channels;

        // if `output` is empty, there's nothing to fill.
        if output.is_empty() {
            return model;
        }

        // Fill the given buffer with silence.
        fn silence<S: Sample>(buffer: &mut [S]) {
            for sample in buffer {
                *sample = S::equilibrium();
            }
        }

        // Zero the buffer before doing anything else.
        silence(output);

        // Have to have a positive sample_rate or nothing will happen!
        assert!(sample_rate > 0);

        // The starting index of the output slice we'll write to.
        let mut start = 0;

        // Write the contents of b to a.
        fn write<S: Copy>(a: &mut [S], b: &[S]) {
            for (a_sample, b_sample) in a.iter_mut().zip(b) {
                *a_sample = *b_sample;
            }
        }

        // If there is some un-read range of `samples`, read those first.
        if let Some(range) = pending_range.take() {
            // If the pending range would not fill the output, write what we can before going on to
            // request more frames.
            if range.len() < output.len() {
                start = range.len();
                write(&mut output[..range.len()], &samples[range]);

            // If we have the exact number of frames as output, write them and return.
            } else if range.len() == output.len() {
                write(output, &samples[range]);
                return model;
            } else {
                let end = range.start + output.len();
                write(output, &samples[range.start..end]);
                *pending_range = Some(end..range.end);
                return model;
            }
        }

        // Ensure that our buffer has `num_frames` `frames`.
        samples.resize(num_samples, S::equilibrium());

        // Loop until the given `output` is filled.
        loop {
            // See how many frames are left to fill.
            let num_samples_remaining = output.len() - start;

            // The number of frames to write to output on this iteration.
            let num_samples_to_fill = std::cmp::min(samples.len(), num_samples_remaining);

            // Zero the `samples` buffer ready for summing.
            silence(samples);

            // Render the state of the model to the samples buffer.
            let interleaved_samples = std::mem::replace(samples, Vec::new()).into_boxed_slice();
            let mut buffer = audio::Buffer {
                interleaved_samples,
                channels,
                sample_rate,
            };
            render(&mut model, &mut buffer);
            let mut new_samples = buffer.interleaved_samples.into_vec();
            std::mem::swap(samples, &mut new_samples);

            // Write the `frames` to output.
            let end = start + num_samples_to_fill;
            let range = start..end;
            write(&mut output[range.clone()], &samples[..range.len()]);

            // If this was the last frame, break from the loop.
            if end == output.len() {
                // If this is the last iteration and not all of `frames` were read, store the
                // `pending_range` to be read next time this method is called.
                if range.len() < samples.len() {
                    *pending_range = Some(range.len()..samples.len());
                }

                break;
            }

            // Continue looping through the next frames.
            start = end;
        }

        model
    }
}
