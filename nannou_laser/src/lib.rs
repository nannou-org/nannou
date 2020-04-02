//! A cross-platform laser DAC detection and streaming API.

pub extern crate ether_dream;

pub mod dac;
pub mod ffi;
pub mod lerp;
pub mod point;
pub mod stream;
pub mod util;

pub use dac::{DetectDacs, DetectDacsAsync, DetectedDac, DetectedDacCallback, Id as DacId};
pub use lerp::Lerp;
pub use point::{Point, RawPoint};
pub use stream::frame::Frame;
pub use stream::frame::Stream as FrameStream;
pub use stream::raw::Buffer;
pub use stream::raw::Stream as RawStream;

use std::io;
use std::sync::Arc;
use std::time::Duration;

/// A general API that allows for detecting and enumerating laser DACs on a network and
/// establishing new streams of communication with them.
pub struct Api {
    inner: Arc<Inner>,
}

// The inner state of the `Api` that can be easily shared between laser threads in an `Arc`.
//
// This is useful for allowing streams to re-scan and find their associated DAC in the case it
// drops out for some reason.
pub(crate) struct Inner;

impl Api {
    /// Instantiate the laser API.
    pub fn new() -> Self {
        Api {
            inner: Arc::new(Inner),
        }
    }

    /// An iterator yielding laser DACs available on the system as they are discovered.
    ///
    /// Currently, the only laser protocol supported is the ether dream protocol. Thus, this
    /// enumerates ether dream DACs that are discovered on the LAN.
    ///
    /// **Note** that the produced iterator will iterate forever and never terminate unless
    /// `set_timeout` is called on the returned `DetectDacs` instance.
    pub fn detect_dacs(&self) -> io::Result<DetectDacs> {
        self.inner.detect_dacs()
    }

    /// Block and wait until the DAC with the given `Id` is detected.
    pub fn detect_dac(&self, id: DacId) -> io::Result<DetectedDac> {
        self.inner.detect_dac(id)
    }

    /// Spawn a thread for DAC detection.
    ///
    /// Calls the given `callback` with broadcasts as they are received.
    ///
    /// The thread is closed when the returned `DetectDacsAsync` instance is dropped.
    pub fn detect_dacs_async<F>(
        &self,
        timeout: Option<Duration>,
        callback: F,
    ) -> io::Result<DetectDacsAsync>
    where
        F: 'static + DetectedDacCallback + Send,
    {
        self.inner.detect_dacs_async(timeout, callback)
    }

    /// Begin building a new laser frame stream.
    ///
    /// The stream will call the `render` function each time new points are needed to feed the
    /// laser DAC buffer. The rate at which this will be called depends on the `point_hz`,
    /// `frame_hz` and the `latency_points`.
    pub fn new_frame_stream<M, F>(&self, model: M, render: F) -> stream::frame::Builder<M, F>
    where
        F: stream::frame::RenderFn<M>,
    {
        let api_inner = self.inner.clone();
        let builder = Default::default();
        let frame_hz = None;
        let interpolation_conf = Default::default();
        let process_raw = stream::frame::default_process_raw_fn;
        stream::frame::Builder {
            api_inner,
            builder,
            model,
            render,
            process_raw,
            frame_hz,
            interpolation_conf,
        }
    }

    /// Begin building a new laser raw stream.
    ///
    /// The raw stream will call the given `render` function with a request for as many points as
    /// the DAC currently might need to fill the buffer based on the stream latency.
    pub fn new_raw_stream<M, F>(&self, model: M, render: F) -> stream::raw::Builder<M, F>
    where
        F: stream::raw::RenderFn<M>,
    {
        let api_inner = self.inner.clone();
        let builder = Default::default();
        stream::raw::Builder {
            api_inner,
            builder,
            model,
            render,
        }
    }
}

impl Inner {
    /// See the `Api::detect_dacs` docs.
    pub(crate) fn detect_dacs(&self) -> io::Result<DetectDacs> {
        dac::detect_dacs()
    }

    /// Block and wait until the DAC with the given `Id` is detected.
    pub(crate) fn detect_dac(&self, id: DacId) -> io::Result<DetectedDac> {
        for res in self.detect_dacs()? {
            let dac = res?;
            if dac.id() == id {
                return Ok(dac);
            }
        }
        unreachable!("DAC detection iterator should never return `None`")
    }

    /// See the `Api::detect_dacs_async` docs.
    fn detect_dacs_async<F>(
        &self,
        timeout: Option<Duration>,
        callback: F,
    ) -> io::Result<DetectDacsAsync>
    where
        F: 'static + DetectedDacCallback + Send,
    {
        dac::detect_dacs_async(timeout, callback)
    }
}

impl AsRef<Point> for Point {
    fn as_ref(&self) -> &Point {
        self
    }
}
