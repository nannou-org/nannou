//! A cross-platform laser DAC detection and streaming API.

pub extern crate ether_dream;

pub mod lerp;
pub mod point;
pub mod stream;
pub mod util;

pub use lerp::Lerp;
pub use point::{Point, RawPoint};
pub use stream::raw::Buffer;
pub use stream::raw::Stream as RawStream;
pub use stream::frame::Frame;
pub use stream::frame::Stream as FrameStream;

use std::io;
use std::sync::Arc;

/// A general API that allows for detecting and enumerating laser DACs on a network and
/// establishing new streams of communication with them.
pub struct Lasy {
    inner: Arc<Inner>,
}

// The inner state of the `Lasy` that can be easily shared between laser threads in an `Arc`.
//
// This is useful for allowing streams to re-scan and find their associated DAC in the case it
// drops out for some reason.
pub(crate) struct Inner;

/// An iterator yielding laser DACs available on the system as they are discovered.
pub struct DetectDacs {
    dac_broadcasts: ether_dream::RecvDacBroadcasts,
}

/// An available DAC detected on the system.
#[derive(Clone, Debug)]
pub enum DetectedDac {
    /// An ether dream laser DAC discovered via the ether dream protocol broadcast message.
    EtherDream {
        broadcast: ether_dream::protocol::DacBroadcast,
        source_addr: std::net::SocketAddr,
    },
}

/// A persistent, unique identifier associated with a DAC (like a MAC address).
///
/// It should be possible to use this to uniquely identify the same DAC on different occasions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DacId {
    EtherDream {
        mac_address: [u8; 6],
    }
}

impl Lasy {
    /// Instantiate the laser API.
    pub fn new() -> Self {
        Lasy { inner: Arc::new(Inner) }
    }

    /// An iterator yielding laser DACs available on the system as they are discovered.
    ///
    /// Currently, the only laser protocol supported is the ether dream protocol. Thus, this
    /// enumerates ether dream DACs that are discovered on the LAN.
    ///
    /// **Note** that the produced iterator will iterate forever and never terminate, so you may
    /// only want to check a certain number of entries or run this iterator on some other thread.
    pub fn detect_dacs(&self) -> io::Result<DetectDacs> {
        self.inner.detect_dacs()
    }

    /// Block and wait until the DAC with the given `Id` is detected.
    pub fn detect_dac(&self, id: DacId) -> io::Result<DetectedDac> {
        self.inner.detect_dac(id)
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
        stream::raw::Builder { api_inner, builder, model, render }
    }
}

impl Inner {
    // See the `Lasy::detect_dacs` docs.
    pub(crate) fn detect_dacs(&self) -> io::Result<DetectDacs> {
        let dac_broadcasts = ether_dream::recv_dac_broadcasts()?;
        Ok(DetectDacs { dac_broadcasts })
    }

    // Block and wait until the DAC with the given `Id` is detected.
    pub(crate) fn detect_dac(&self, id: DacId) -> io::Result<DetectedDac> {
        for res in self.detect_dacs()? {
            let dac = res?;
            if dac.id() == id {
                return Ok(dac);
            }
        }
        unreachable!("DAC detection iterator should never return `None`")
    }
}

impl DetectedDac {
    /// The maximum point rate allowed by the DAC.
    pub fn max_point_hz(&self) -> u32 {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => broadcast.max_point_rate as _,
        }
    }

    /// The number of points that can be stored within the buffer.
    pub fn buffer_capacity(&self) -> u32 {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => broadcast.buffer_capacity as _,
        }
    }

    /// A persistent, unique identifier associated with the DAC (like a MAC address).
    ///
    /// It should be possible to use this to uniquely identify the same DAC on different occasions.
    pub fn id(&self) -> DacId {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => {
                DacId::EtherDream {
                    mac_address: broadcast.mac_address,
                }
            }
        }
    }
}

impl AsRef<Point> for Point {
    fn as_ref(&self) -> &Point {
        self
    }
}

impl Iterator for DetectDacs {
    type Item = io::Result<DetectedDac>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.dac_broadcasts.next()?;
        match res {
            Err(err) => Some(Err(err)),
            Ok((broadcast, source_addr)) => {
                Some(Ok(DetectedDac::EtherDream { broadcast, source_addr }))
            }
        }
    }
}
