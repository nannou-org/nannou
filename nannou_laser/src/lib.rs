//! A cross-platform laser DAC detection and streaming API.

pub extern crate ether_dream;
pub extern crate helios_dac;

pub mod dac_manager;
pub mod dac_manager_etherdream;
pub mod dac_manager_helios;
#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "ilda-idtf")]
pub mod ilda_idtf;
pub mod point;
pub mod stream;
pub mod util;

pub use dac_manager::{DacVariant, DetectDacs, DetectedDac, DetectedDacError, Id as DacId, Result};
pub use dac_manager_etherdream::{DetectEtherDreamDacsAsync, DetectedDacCallback};
pub use point::{Point, RawPoint};
pub use stream::frame::Frame;
pub use stream::frame::Stream as FrameStream;
pub use stream::raw::Stream as RawStream;
pub use stream::raw::{Buffer, StreamError, StreamErrorAction};

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

    /// An iterator yielding laser DACs available on the system per supported DAC variant.
    ///
    /// **Note** that the DetectDacs::EtherDream iterator will iterate forever and never terminate unless
    /// `set_timeout` is called on the returned `DetectDacs` instance.
    pub fn detect_dacs(&self, variant: DacVariant) -> Result<DetectDacs> {
        self.inner.detect_dacs(variant)
    }

    /// Block and wait until the DAC with the given `Id` is detected.
    pub fn detect_dac(&self, id: DacId, variant: DacVariant) -> Result<DetectedDac> {
        self.inner.detect_dac(id, variant)
    }

    /// Spawn a thread for DAC detection. Currently only implemented for the Etherdream DAC
    ///
    /// Calls the given `callback` with broadcasts as they are received.
    ///
    /// The thread is closed when the returned `DetectDacsAsync` instance is dropped.
    pub fn detect_dacs_async<F>(
        &self,
        timeout: Option<Duration>,
        callback: F,
    ) -> io::Result<DetectEtherDreamDacsAsync>
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
        let enable_optimisations = stream::DEFAULT_ENABLE_OPTIMISATIONS;
        let enable_draw_reorder = stream::DEFAULT_ENABLE_DRAW_REORDER;
        let process_raw = stream::frame::default_process_raw_fn;
        let stream_error = stream::raw::default_stream_error_fn;
        stream::frame::Builder {
            api_inner,
            builder,
            model,
            render,
            process_raw,
            stream_error,
            frame_hz,
            interpolation_conf,
            enable_optimisations,
            enable_draw_reorder,
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
        let stream_error = stream::raw::default_stream_error_fn;
        stream::raw::Builder {
            api_inner,
            builder,
            model,
            render,
            stream_error,
            is_frame: false,
        }
    }
}

impl Inner {
    /// See the `Api::detect_dacs` docs.
    pub(crate) fn detect_dacs(&self, variant: DacVariant) -> Result<DetectDacs> {
        match variant {
            DacVariant::DacVariantEtherdream => {
                dac_manager_etherdream::detect_dacs().map_err(DetectedDacError::from)
            }
            DacVariant::DacVariantHelios => dac_manager_helios::detect_dacs(),
        }
    }

    /// Block and wait until the DAC with the given `Id` is detected.
    pub(crate) fn detect_dac(&self, id: DacId, variant: DacVariant) -> Result<DetectedDac> {
        for res in self.detect_dacs(variant)? {
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
    ) -> io::Result<DetectEtherDreamDacsAsync>
    where
        F: 'static + DetectedDacCallback + Send,
    {
        dac_manager_etherdream::detect_dacs_async(timeout, callback)
    }
}

impl AsRef<Point> for Point {
    fn as_ref(&self) -> &Point {
        self
    }
}
