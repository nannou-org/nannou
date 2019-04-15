use crate::Point;
use crate::lerp::Lerp;
use crate::stream;
use crate::stream::raw::{self, Buffer};
use std::io;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

pub mod opt;

/// The function that will be called each time a new `Frame` is requested.
pub trait RenderFn<M>: Fn(&mut M, &mut Frame) {}
impl<M, F> RenderFn<M> for F where F: Fn(&mut M, &mut Frame) {}

/// A clone-able handle around a laser stream of frames.
pub struct Stream<M> {
    raw: raw::Stream<M>,
}

/// A wrapper around the `Vec` of points being collected for the frame.
///
/// Provides a suite of methods that ease the process of submitting points.
///
/// Segments that contain more than one blank point in a row will be considered a blank segment.
pub struct Frame {
    frame_hz: u32,
    point_hz: u32,
    latency_points: u32,
    points: Vec<Point>,
}

// A type used for requesting frames from the user and feeding them to the raw buffer.
struct Requester {
    last_frame_point: Option<Point>,
    points: Vec<Point>,
}

/// A type allowing to build a raw laser stream.
pub struct Builder<M, F> {
    /// The laser API inner state, used to find a DAC during `build` if one isn't specified.
    pub(crate) api_inner: Arc<crate::Inner>,
    pub builder: stream::Builder,
    pub model: M,
    pub render: F,
    pub frame_hz: Option<u32>,
}

impl<M, F> Builder<M, F> {
    /// The DAC with which the stream should be established.
    pub fn detected_dac(mut self, dac: crate::DetectedDac) -> Self {
        self.builder.dac = Some(dac);
        self
    }

    /// The initial rate at which the DAC should process points per second.
    ///
    /// This value should be no greater than the detected DAC's `max_point_hz`.
    ///
    /// By default this value is `stream::DEFAULT_POINT_HZ`.
    pub fn point_hz(mut self, point_hz: u32) -> Self {
        self.builder.point_hz = Some(point_hz);
        self
    }

    /// The initial rate at which the DAC should output frames per second.
    ///
    /// This in combination with the `point_hz` is used to determine the `points_per_frame`. Frames
    /// yielded by the user will be interpolated so that they always use exactly `points_per_frame`
    /// number of points per frame.
    ///
    /// By default, this value is `stream::DEFAULT_FRAME_HZ`.
    pub fn frame_hz(mut self, frame_hz: u32) -> Self {
        self.frame_hz = Some(frame_hz);
        self
    }

    /// The maximum latency specified as a number of points.
    ///
    /// Each time the laser indicates its "fullness", the raw stream will request enough points
    /// from the render function to fill the DAC buffer up to `latency_points`.
    pub fn latency_points(mut self, points: u32) -> Self {
        self.builder.latency_points = Some(points);
        self
    }

    /// Build the stream with the specified parameters.
    ///
    /// **Note:** If no `dac` was specified, this will method will block until a DAC is detected.
    /// The first detected DAC is the DAC with which a stream will be established.
    pub fn build(self) -> io::Result<Stream<M>>
    where
        M: 'static + Send,
        F: 'static + RenderFn<M> + Send,
    {
        let Builder { api_inner, builder, model, render, frame_hz } = self;

        // Retrieve the frame rate to initialise the stream with.
        let frame_hz = frame_hz.unwrap_or(stream::DEFAULT_FRAME_HZ);

        // The type used for buffering frames and using them to serve points to the raw stream.
        let requester = Arc::new(Mutex::new(Requester { last_frame_point: None, points: vec![] }));

        // A render function for the inner raw stream.
        let raw_render = move |model: &mut M, buffer: &mut Buffer| {
            let mut guard = requester.lock().expect("failed to lock frame requester");
            guard.fill_buffer(model, &render, buffer, frame_hz);
        };

        // Create the raw builder and build the raw stream.
        let raw_builder = raw::Builder { api_inner, builder, model, render: raw_render };
        let raw_stream = raw_builder.build()?;
        let stream = Stream { raw: raw_stream };
        Ok(stream)
    }
}

impl Frame {
    /// The rate at which frames of points will be emitted by the DAC.
    pub fn frame_hz(&self) -> u32 {
        self.frame_hz
    }

    /// The rate at which these points will be emitted by the DAC.
    pub fn point_hz(&self) -> u32 {
        self.point_hz
    }

    /// The maximum number of points with which to fill the DAC's buffer.
    pub fn latency_points(&self) -> u32 {
        self.latency_points
    }

    /// The number of points emitted by the DAC per frame.
    pub fn points_per_frame(&self) -> u32 {
        self.point_hz / self.frame_hz
    }

    /// Add a sequence of consecutive points.
    ///
    /// If some points already exist in the frame, this method will create a blank segment between
    /// the previous point and the first point before appending this sequence.
    pub fn add_points<I>(&mut self, points: I)
    where
        I: IntoIterator,
        I::Item: AsRef<Point>,
    {
        let mut points = points.into_iter();
        if let Some(&last) = self.points.last() {
            if let Some(next) = points.next() {
                let next = next.as_ref();
                self.points.push(last.blanked());
                self.points.push(next.blanked());
                self.points.push(*next);
            }
        }
        self.points.extend(points.map(|p| p.as_ref().clone()));
    }
}

impl Requester {
    // Fill the given buffer by requesting frames from the given user `render` function as
    // required.
    fn fill_buffer<M, F>(
        &mut self,
        model: &mut M,
        render: F,
        buffer: &mut Buffer,
        frame_hz: u32,
    )
    where
        F: RenderFn<M>,
    {
        // If the frame rate is `0`, leave the buffer empty.
        if frame_hz == 0 {
            return;
        }

        // If the buffer has no points, there's nothing to fill.
        if buffer.is_empty() {
            return;
        }

        // The number of points to generate per frame.
        let point_hz = buffer.point_hz();
        let latency_points = buffer.latency_points();

        // The starting index of the buffer we'll write to.
        let mut start = 0;

        // If there are still un-read points, use those first.
        if !self.points.is_empty() {
            // If the pending range would not fill the buffer, write what we can.
            if self.points.len() < buffer.len() {
                start = self.points.len();
                buffer[..start].copy_from_slice(&self.points);
                self.points.clear();

            // If we have the exact number of frames as output, write them and return.
            } else if self.points.len() == buffer.len() {
                buffer.copy_from_slice(&self.points);
                self.points.clear();
                return;

            // If we have too many points, write what we can and leave the rest.
            } else {
                let end = buffer.len();
                buffer.copy_from_slice(&self.points[..end]);
                self.points.drain(0..end);
                return;
            }
        }

        // The number of points to fill for each frame.
        let points_per_frame = point_hz / frame_hz;

        // If we reached this point, `self.points` is empty so we should fill buffer with frames
        // until it is full.
        loop {
            // See how many points are left to fill.
            let num_points_remaining = buffer.len() - start;

            // Determine how many points to fill this pass.
            let num_points_to_fill = std::cmp::min(points_per_frame as usize, num_points_remaining);

            // Render a frame of points.
            let points = std::mem::replace(&mut self.points, Vec::new());
            let mut frame = Frame {
                point_hz,
                latency_points,
                frame_hz,
                points,
            };
            render(model, &mut frame);

            // If we were given no points, back to the top of the loop to collect more.
            if frame.points.is_empty() {
                continue;
            }

            // The buffer range to fill.
            let end = start + num_points_to_fill;
            let range = start..end;

            // TODO: Remove this and replace with optimiser and proper interpolater.
            {
                // Blank from last of prev frame to first of next.
                if let Some(last) = self.last_frame_point.take() {
                    let a = last.blanked();
                    let b = frame.points[0].blanked();
                    frame.insert(0, b);
                    frame.insert(0, a);
                }

                // Assign the last frame point.
                self.last_frame_point = frame.last().map(|&p| p);

                // Lerp the frame points into the requester's points buffer.
                for i in 0..points_per_frame {
                    let i_fract = i as f32 / points_per_frame as f32;
                    let point_lerp = i_fract * (frame.points.len() - 1) as f32;
                    let ix_a = point_lerp as usize;
                    let ix_b = ix_a + 1;
                    let a = frame.points[ix_a];
                    let b = &frame.points[ix_b];
                    let lerp_amt = point_lerp.fract();
                    let p = a.lerp(b, lerp_amt);
                    self.points.push(p);
                }

                buffer[range.clone()].copy_from_slice(&self.points[..range.len()]);
                self.points.drain(..range.len());
            }

            // If this output filled the buffer, break.
            if end == buffer.len() {
                break;
            }

            // Continue looping through the next frames.
            start = end;
        }
    }
}

impl Deref for Frame {
    type Target = Vec<Point>;
    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.points
    }
}

impl<M> Deref for Stream<M> {
    type Target = raw::Stream<M>;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
