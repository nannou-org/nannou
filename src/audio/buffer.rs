use std;
use std::ops::{Deref, DerefMut};

pub struct Buffer<S=f32> {
    pub (crate) interleaved_samples: Box<[S]>,
    pub (crate) channels: usize,
    pub (crate) sample_rate: u32,
}

/// An iterator yielding each frame in some Buffer.
///
/// A "frame" is a sample from each channel of audio at a single moment in time.
#[derive(Clone)]
pub struct Frames<'a, S: 'a> {
    chunks: std::slice::Chunks<'a, S>,
}

/// An iterator yielding mutable references to each frame in some Buffer.
///
/// A frame is a sample from each channel of audio at a single moment in time.
pub struct FramesMut<'a, S: 'a> {
    chunks: std::slice::ChunksMut<'a, S>,
}

impl<'a, S> Iterator for Frames<'a, S> {
    type Item = &'a [S];
    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
    }
}

impl<'a, S> Iterator for FramesMut<'a, S> {
    type Item = &'a mut [S];
    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
    }
}

impl<S> Buffer<S> {
    /// The default number of frames per buffer.
    ///
    /// This can be overridden by specifying your own buffer size using the
    /// `stream::output::Builder::buffer_size` builder method.
    pub const DEFAULT_LEN_FRAMES: usize = 64;

    /// The sampling rate of the audio frames stored within the buffer.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// The number of channels of audio per-frame within the buffer.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// The length of the buffer as a number of audio frames (i.e. len / channels).
    pub fn len_frames(&self) -> usize {
        self.interleaved_samples.len() / self.channels
    }

    /// Produce an iterator yielding each frame from the buffer in order.
    pub fn frames(&self) -> Frames<S> {
        let chunks = self.interleaved_samples.chunks(self.channels);
        Frames { chunks }
    }

    pub fn frames_mut(&mut self) -> FramesMut<S> {
        let chunks = self.interleaved_samples.chunks_mut(self.channels);
        FramesMut { chunks }
    }
}

impl<S> Deref for Buffer<S> {
    type Target = [S];
    fn deref(&self) -> &Self::Target {
        &self.interleaved_samples
    }
}

impl<S> DerefMut for Buffer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interleaved_samples
    }
}
